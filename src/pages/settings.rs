use leptos::prelude::*;

use crate::models::MixChoice;
use crate::server_fns::{
    get_paint_brands, get_paint_colors, get_user_paint_settings, save_user_paint_settings,
    PaintColorInfo,
};

const DEFAULT_BRAND: &str = "michael_harding";

#[component]
pub fn SettingsPage() -> impl IntoView {
    let brands = Resource::new(|| (), |_| get_paint_brands());
    let settings = Resource::new(|| (), |_| get_user_paint_settings());

    let (selected_brand, set_selected_brand) = signal(DEFAULT_BRAND.to_string());
    let (selected_colors, set_selected_colors) = signal(Vec::<String>::new());
    let (mix_choice, set_mix_choice) = signal("black + white + 2 colours".to_string());
    let (save_status, set_save_status) = signal(Option::<String>::None);
    let (initialized, set_initialized) = signal(false);
    let (user_has_interacted, set_user_has_interacted) = signal(false);

    // Load colors when brand changes
    let colors = Resource::new(
        move || selected_brand.get(),
        |brand| async move {
            if brand.is_empty() {
                Ok(vec![])
            } else {
                get_paint_colors(brand).await
            }
        },
    );

    // Initialize from saved settings or use defaults
    Effect::new(move || {
        if let Some(Ok(s)) = settings.get() {
            if !s.brand.is_empty() {
                set_selected_brand.set(s.brand);
                if !s.colors.is_empty() {
                    set_selected_colors.set(s.colors);
                }
            }
            if !s.mix_choice.is_empty() {
                set_mix_choice.set(s.mix_choice);
            }
            set_initialized.set(true);
        }
    });

    // Auto-select all colors when colors load and user has no saved settings
    // But only if user hasn't manually interacted with the toggle
    Effect::new(move || {
        if let Some(Ok(color_list)) = colors.get() {
            // Only auto-select all if this is fresh (no saved colors), we just initialized,
            // and user hasn't manually toggled
            if initialized.get()
                && !user_has_interacted.get()
                && selected_colors.get().is_empty()
                && !color_list.is_empty()
            {
                let all_ids: Vec<String> = color_list.iter().map(|c| c.id.clone()).collect();
                set_selected_colors.set(all_ids);
            }
        }
    });

    // Store available colors for toggle all functionality
    let (available_colors, set_available_colors) = signal(Vec::<PaintColorInfo>::new());
    Effect::new(move || {
        if let Some(Ok(color_list)) = colors.get() {
            set_available_colors.set(color_list);
        }
    });

    // Check if all colors are selected
    let all_selected = move || {
        let available = available_colors.get();
        let selected = selected_colors.get();
        !available.is_empty() && available.len() == selected.len()
    };

    let save_settings = Action::new(move |_: &()| {
        let brand = selected_brand.get();
        let colors = selected_colors.get();
        let choice = mix_choice.get();

        async move {
            set_save_status.set(Some("Saving...".to_string()));
            match save_user_paint_settings(choice, brand, colors).await {
                Ok(()) => set_save_status.set(Some("Settings saved!".to_string())),
                Err(e) => set_save_status.set(Some(format!("Error: {}", e))),
            }
        }
    });

    let toggle_color = move |color: String| {
        set_selected_colors.update(|colors| {
            if colors.contains(&color) {
                colors.retain(|c| c != &color);
            } else {
                colors.push(color);
            }
        });
    };

    view! {
        <div class="settings-page">
            <h1>"Paint Settings"</h1>
            <p class="subtitle">"Configure your paint palette and mixing preferences"</p>

            <div class="settings-section">
                <h2>"Mix Strategy"</h2>
                <select
                    class="select-input"
                    on:change=move |ev| {
                        set_mix_choice.set(event_target_value(&ev));
                    }
                    prop:value=move || mix_choice.get()
                >
                    {MixChoice::all()
                        .into_iter()
                        .map(|choice| {
                            let value = choice.as_str();
                            view! {
                                <option value=value selected=move || mix_choice.get() == value>
                                    {value}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
            </div>

            <div class="settings-section">
                <h2>"Paint Brand"</h2>
                <Suspense fallback=move || view! { <p>"Loading brands..."</p> }>
                    {move || {
                        brands
                            .get()
                            .map(|result| {
                                match result {
                                    Ok(brand_list) => {
                                        view! {
                                            <select
                                                class="select-input"
                                                on:change=move |ev| {
                                                    let brand = event_target_value(&ev);
                                                    set_selected_brand.set(brand);
                                                    set_selected_colors.set(vec![]);
                                                }
                                                prop:value=move || selected_brand.get()
                                            >
                                                <option value="">"Select a brand..."</option>
                                                {brand_list
                                                    .into_iter()
                                                    .map(|b| {
                                                        let id = b.id.clone();
                                                        view! {
                                                            <option
                                                                value=b.id.clone()
                                                                selected=move || selected_brand.get() == id
                                                            >
                                                                {b.name}
                                                            </option>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </select>
                                        }
                                            .into_any()
                                    }
                                    Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                }
                            })
                    }}
                </Suspense>
            </div>

            <div class="settings-section">
                <h2>"Select Colours"</h2>
                <div class="colour-controls">
                    <p class="hint">
                        "Click colours to add them to your palette. Selected: "
                        {move || selected_colors.get().len()}
                        " / "
                        {move || available_colors.get().len()}
                    </p>
                    <button
                        class="btn toggle-all"
                        on:click=move |_| {
                            set_user_has_interacted.set(true);
                            let available = available_colors.get();
                            if all_selected() {
                                set_selected_colors.set(vec![]);
                            } else {
                                let all_ids: Vec<String> = available.iter().map(|c| c.id.clone()).collect();
                                set_selected_colors.set(all_ids);
                            }
                        }
                        disabled=move || available_colors.get().is_empty()
                    >
                        {move || if all_selected() { "Deselect All" } else { "Select All" }}
                    </button>
                </div>
                <Suspense fallback=move || view! { <p>"Loading colours..."</p> }>
                    {move || {
                        colors
                            .get()
                            .map(|result| {
                                match result {
                                    Ok(color_list) => {
                                        if color_list.is_empty() {
                                            view! { <p class="hint">"Select a brand first"</p> }
                                                .into_any()
                                        } else {
                                            view! {
                                                <div class="colour-grid">
                                                    {color_list
                                                        .into_iter()
                                                        .map(|c| {
                                                            let id = c.id.clone();
                                                            let id2 = c.id.clone();
                                                            let hex = c.hex.clone();
                                                            view! {
                                                                <button
                                                                    class="colour-swatch"
                                                                    class:selected=move || {
                                                                        selected_colors.get().contains(&id)
                                                                    }
                                                                    style=format!("background-color: {}", hex)
                                                                    title=id2.clone()
                                                                    on:click=move |_| toggle_color(id2.clone())
                                                                >
                                                                    <span class="colour-name">{c.id}</span>
                                                                </button>
                                                            }
                                                        })
                                                        .collect_view()}
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    }
                                    Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                }
                            })
                    }}
                </Suspense>
            </div>

            <div class="settings-actions">
                <button class="btn primary" on:click=move |_| { save_settings.dispatch(()); }>
                    "Save Settings"
                </button>
                {move || {
                    save_status
                        .get()
                        .map(|status| {
                            view! { <span class="save-status">{status}</span> }
                        })
                }}
            </div>
        </div>
    }
}
