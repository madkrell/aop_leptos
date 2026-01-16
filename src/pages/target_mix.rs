use leptos::prelude::*;
use leptos::web_sys;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

use crate::models::MixingResult;
use crate::server_fns::find_paint_mix;

#[derive(Clone, Copy, PartialEq)]
enum InputMode {
    Picker,
    Image,
}

#[component]
pub fn TargetMixPage() -> impl IntoView {
    let (target_colour, set_target_colour) = signal("#808080".to_string());
    let (r, g, b) = (signal(128u8), signal(128u8), signal(128u8));
    let (results, set_results) = signal(Option::<Vec<MixingResult>>::None);
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    // Input mode: colour picker or image (default to image)
    let (input_mode, set_input_mode) = signal(InputMode::Image);

    // Image state - simplified: just the source, no custom zoom/pan
    let (image_src, set_image_src) = signal(Option::<String>::None);

    // Update RGB from hex
    let update_from_hex = move |hex: String| {
        if hex.len() == 7 && hex.starts_with('#') {
            if let (Ok(red), Ok(green), Ok(blue)) = (
                u8::from_str_radix(&hex[1..3], 16),
                u8::from_str_radix(&hex[3..5], 16),
                u8::from_str_radix(&hex[5..7], 16),
            ) {
                r.1.set(red);
                g.1.set(green);
                b.1.set(blue);
            }
        }
        set_target_colour.set(hex);
    };

    // Update hex from RGB
    let update_hex = move || {
        let hex = format!("#{:02x}{:02x}{:02x}", r.0.get(), g.0.get(), b.0.get());
        set_target_colour.set(hex);
    };

    // Handle image file selection
    #[allow(unused_variables)]
    let handle_image_upload = move |ev: web_sys::Event| {
        #[cfg(feature = "hydrate")]
        {
            use ::web_sys::{FileReader, HtmlInputElement};

            let input = ev.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let reader = FileReader::new().unwrap();
                    let reader_clone = reader.clone();

                    let closure = wasm_bindgen::closure::Closure::once(Box::new(move || {
                        if let Ok(result) = reader_clone.result() {
                            if let Some(data_url) = result.as_string() {
                                set_image_src.set(Some(data_url));
                                set_input_mode.set(InputMode::Image);
                            }
                        }
                    }) as Box<dyn FnOnce()>);

                    reader.set_onload(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();

                    let _ = reader.read_as_data_url(&file);
                }
            }
        }
        let _ = ev;
    };

    // Handle clicking on image to pick colour
    // Simple approach: use offsetX/offsetY which gives position relative to the element
    #[allow(unused_variables)]
    let handle_image_click = move |ev: web_sys::MouseEvent| {
        #[cfg(feature = "hydrate")]
        {
            use ::web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

            let target = ev.target().unwrap();
            let img = target.dyn_into::<HtmlImageElement>().unwrap();

            // Get natural (original) dimensions of the image
            let natural_width = img.natural_width();
            let natural_height = img.natural_height();

            // Get the displayed dimensions
            let displayed_width = img.width() as f64;
            let displayed_height = img.height() as f64;

            if displayed_width == 0.0 || displayed_height == 0.0 || natural_width == 0 || natural_height == 0 {
                return;
            }

            // Use offsetX/offsetY - position relative to the clicked element
            let offset_x = ev.offset_x() as f64;
            let offset_y = ev.offset_y() as f64;

            // Scale from displayed size to natural size
            let scale_x = natural_width as f64 / displayed_width;
            let scale_y = natural_height as f64 / displayed_height;

            let x = (offset_x * scale_x).round() as u32;
            let y = (offset_y * scale_y).round() as u32;

            // Clamp to valid range
            let x = x.min(natural_width - 1);
            let y = y.min(natural_height - 1);

            // Create canvas at natural size for accurate sampling
            let document = ::web_sys::window().unwrap().document().unwrap();
            let canvas = document
                .create_element("canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();
            canvas.set_width(natural_width);
            canvas.set_height(natural_height);

            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            // Draw image at natural size
            let _ = ctx.draw_image_with_html_image_element(&img, 0.0, 0.0);

            // Sample pixel at calculated position
            if let Ok(image_data) = ctx.get_image_data(x as f64, y as f64, 1.0, 1.0) {
                let data = image_data.data();
                let red = data[0];
                let green = data[1];
                let blue = data[2];

                r.1.set(red);
                g.1.set(green);
                b.1.set(blue);
                update_hex();
            }
        }
        let _ = ev;
    };

    let find_mix = Action::new(move |_: &()| {
        let red = r.0.get();
        let green = g.0.get();
        let blue = b.0.get();

        async move {
            set_loading.set(true);
            set_error.set(None);
            set_results.set(None);

            match find_paint_mix(red, green, blue).await {
                Ok(res) => {
                    set_results.set(Some(res));
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        }
    });

    view! {
        <div class="target-mix-page">
            <div class="page-header">
                <h1>"Target Colour Mix"</h1>
                <p class="subtitle">"Select a target colour and find the optimal paint mixture"</p>
            </div>

            // Top control bar: mode selector, selected colour, and find mix button
            <div class="top-controls">
                <div class="mode-selector">
                    <button
                        class="mode-btn"
                        class:active=move || input_mode.get() == InputMode::Picker
                        on:click=move |_| set_input_mode.set(InputMode::Picker)
                    >
                        "Colour Picker"
                    </button>
                    <button
                        class="mode-btn"
                        class:active=move || input_mode.get() == InputMode::Image
                        on:click=move |_| set_input_mode.set(InputMode::Image)
                    >
                        "Image Upload"
                    </button>
                </div>

                <div class="selected-colour-display">
                    <div
                        class="colour-swatch"
                        style=move || format!("background-color: {}", target_colour.get())
                    ></div>
                    <span class="colour-value">{move || target_colour.get()}</span>
                    <span class="colour-rgb">
                        {move || format!("RGB({}, {}, {})", r.0.get(), g.0.get(), b.0.get())}
                    </span>
                </div>

                <button
                    class="btn primary find-mix-btn"
                    on:click=move |_| { find_mix.dispatch(()); }
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Finding..." } else { "Find Mix" }}
                </button>
            </div>

            // Main content area
            <div class="main-content" class:has-results=move || results.get().is_some()>
                // Left panel: Input (picker or image)
                <div class="input-panel" class:image-mode=move || input_mode.get() == InputMode::Image>
                    {move || match input_mode.get() {
                        InputMode::Picker => {
                            view! {
                                <div class="picker-section">
                                    <div
                                        class="colour-preview"
                                        style=move || format!("background-color: {}", target_colour.get())
                                    >
                                        <span class="colour-hex">{move || target_colour.get()}</span>
                                    </div>

                                    <div class="colour-inputs">
                                        <div class="input-group">
                                            <label>"Colour Picker"</label>
                                            <input
                                                type="color"
                                                prop:value=move || target_colour.get()
                                                on:input=move |ev| update_from_hex(event_target_value(&ev))
                                            />
                                        </div>

                                        <div class="input-group">
                                            <label>"Hex"</label>
                                            <input
                                                type="text"
                                                prop:value=move || target_colour.get()
                                                on:input=move |ev| update_from_hex(event_target_value(&ev))
                                                placeholder="#808080"
                                            />
                                        </div>

                                        <div class="rgb-inputs">
                                            <div class="input-group">
                                                <label>"R"</label>
                                                <input
                                                    type="number"
                                                    min="0"
                                                    max="255"
                                                    prop:value=move || r.0.get().to_string()
                                                    on:input=move |ev| {
                                                        if let Ok(v) = event_target_value(&ev).parse() {
                                                            r.1.set(v);
                                                            update_hex();
                                                        }
                                                    }
                                                />
                                            </div>
                                            <div class="input-group">
                                                <label>"G"</label>
                                                <input
                                                    type="number"
                                                    min="0"
                                                    max="255"
                                                    prop:value=move || g.0.get().to_string()
                                                    on:input=move |ev| {
                                                        if let Ok(v) = event_target_value(&ev).parse() {
                                                            g.1.set(v);
                                                            update_hex();
                                                        }
                                                    }
                                                />
                                            </div>
                                            <div class="input-group">
                                                <label>"B"</label>
                                                <input
                                                    type="number"
                                                    min="0"
                                                    max="255"
                                                    prop:value=move || b.0.get().to_string()
                                                    on:input=move |ev| {
                                                        if let Ok(v) = event_target_value(&ev).parse() {
                                                            b.1.set(v);
                                                            update_hex();
                                                        }
                                                    }
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                        InputMode::Image => {
                            view! {
                                <div class="image-section">
                                    // Upload area or image display
                                    {move || match image_src.get() {
                                        None => {
                                            view! {
                                                <label class="image-upload-area">
                                                    <input
                                                        type="file"
                                                        accept="image/*"
                                                        on:change=handle_image_upload
                                                    />
                                                    <div class="upload-placeholder">
                                                        <span class="upload-icon">"📷"</span>
                                                        <span>"Click or drag to upload an image"</span>
                                                        <span class="upload-hint">
                                                            "Click anywhere on the image to sample colours. Use browser zoom (Cmd/Ctrl + scroll) for detail."
                                                        </span>
                                                    </div>
                                                </label>
                                            }
                                                .into_any()
                                        }
                                        Some(src) => {
                                            view! {
                                                <div class="image-container">
                                                    <div class="image-toolbar">
                                                        <span class="zoom-hint">
                                                            "Click to sample colour. Use browser zoom for detail."
                                                        </span>
                                                        <button
                                                            class="tool-btn remove"
                                                            on:click=move |_| set_image_src.set(None)
                                                            title="Remove Image"
                                                        >
                                                            "× Remove"
                                                        </button>
                                                    </div>
                                                    <div class="image-display">
                                                        <img
                                                            src=src
                                                            on:click=handle_image_click
                                                            crossorigin="anonymous"
                                                            draggable="false"
                                                            style="cursor: crosshair; max-width: 100%;"
                                                        />
                                                    </div>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    }}
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </div>

                // Right panel: Results
                {move || {
                    let has_results = results.get().is_some();

                    if !has_results {
                        return None;
                    }

                    Some(
                        view! {
                            <div class="results-panel">
                                // Error message
                                {move || {
                                    error.get().map(|e| view! { <div class="error-message">{e}</div> })
                                }}

                                // Results list
                                {move || {
                                    results
                                        .get()
                                        .map(|res| {
                                            if res.is_empty() {
                                                view! {
                                                    <p class="no-results">"No suitable mixtures found"</p>
                                                }
                                                    .into_any()
                                            } else {
                                                view! {
                                                    <div class="results-content">
                                                        <h2>"Recommended Mixtures"</h2>
                                                        <div class="mix-results">
                                                            {res
                                                                .into_iter()
                                                                .enumerate()
                                                                .map(|(i, mix)| {
                                                                    view! { <MixResultCard mix=mix rank=i + 1 /> }
                                                                })
                                                                .collect_view()}
                                                        </div>
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                        })
                                }}
                            </div>
                        },
                    )
                }}
            </div>
        </div>
    }
}

#[component]
fn MixResultCard(mix: MixingResult, rank: usize) -> impl IntoView {
    let total_weight: f64 = mix.weights.iter().sum();

    view! {
        <div class="mix-result-card">
            <div class="card-header">
                <span class="mix-rank">{"#"}{rank}</span>
                <span class="mix-error">"ΔE: "{format!("{:.2}", mix.error)}</span>
            </div>

            // Horizontal bar chart showing paint proportions
            <div class="mix-bar-chart">
                {mix
                    .weights
                    .iter()
                    .zip(mix.hex_colors.iter())
                    .zip(mix.paints.iter())
                    .map(|((weight, hex), name)| {
                        let percentage = (weight / total_weight * 100.0).round();
                        view! {
                            <div
                                class="bar-segment"
                                style=format!("background-color: {}; width: {}%;", hex, percentage)
                                title=format!("{}: {}%", name, percentage)
                            >
                                {(percentage >= 10.0).then(|| format!("{}%", percentage))}
                            </div>
                        }
                    })
                    .collect_view()}
            </div>

            // Paint list with details
            <div class="paint-details">
                {mix
                    .paints
                    .iter()
                    .zip(mix.weights.iter())
                    .zip(mix.hex_colors.iter())
                    .map(|((name, weight), hex)| {
                        let percentage = (weight / total_weight * 100.0).round();
                        view! {
                            <div class="paint-row">
                                <div
                                    class="paint-swatch"
                                    style=format!("background-color: {}", hex)
                                ></div>
                                <span class="paint-name">{name.clone()}</span>
                                <span class="paint-percentage">{percentage}"%"</span>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}
