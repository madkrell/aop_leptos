use leptos::prelude::*;

use crate::server_fns::{get_user_paint_settings, test_paint_mix, get_paint_colors};

#[component]
pub fn TestMixPage() -> impl IntoView {
    let settings = Resource::new(|| (), |_| get_user_paint_settings());
    let (selected_paints, set_selected_paints) = signal(Vec::<(String, f64)>::new());
    let (result_color, set_result_color) = signal(Option::<String>::None);
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    // Track the brand from settings
    let (current_brand, set_current_brand) = signal(String::new());

    // Update brand when settings load
    Effect::new(move || {
        if let Some(Ok(s)) = settings.get() {
            if !s.brand.is_empty() {
                set_current_brand.set(s.brand);
            }
        }
    });

    // Load colors based on current brand (reactive signal, not resource read)
    let colors = Resource::new(
        move || current_brand.get(),
        |brand| async move {
            if brand.is_empty() {
                Ok(vec![])
            } else {
                get_paint_colors(brand).await
            }
        },
    );

    let add_paint = move |paint: String| {
        set_selected_paints.update(|paints| {
            if !paints.iter().any(|(p, _)| p == &paint) {
                paints.push((paint, 1.0));
            }
        });
    };

    let remove_paint = move |paint: String| {
        set_selected_paints.update(|paints| {
            paints.retain(|(p, _)| p != &paint);
        });
    };

    let update_weight = move |paint: String, weight: f64| {
        set_selected_paints.update(|paints| {
            if let Some((_, w)) = paints.iter_mut().find(|(p, _)| p == &paint) {
                *w = weight;
            }
        });
    };

    let calculate_mix = Action::new(move |_: &()| {
        let paints = selected_paints.get();

        async move {
            if paints.len() < 2 {
                set_error.set(Some("Select at least 2 paints to mix".to_string()));
                return;
            }

            set_loading.set(true);
            set_error.set(None);

            let paint_names: Vec<String> = paints.iter().map(|(p, _)| p.clone()).collect();
            let weights: Vec<f64> = paints.iter().map(|(_, w)| *w).collect();

            match test_paint_mix(paint_names, weights).await {
                Ok(hex) => set_result_color.set(Some(hex)),
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        }
    });

    view! {
        <div class="test-mix-page">
            <h1>"Test Paint Mix"</h1>
            <p class="subtitle">"Create custom paint mixtures and preview the result"</p>

            <div class="mix-builder">
                <div class="available-paints">
                    <h2>"Available Paints"</h2>
                    <Suspense fallback=move || view! { <p>"Loading paints..."</p> }>
                        {move || {
                            colors
                                .get()
                                .map(|result| {
                                    match result {
                                        Ok(color_list) => {
                                            if color_list.is_empty() {
                                                view! {
                                                    <p class="hint">
                                                        "Configure your paint palette in Settings first"
                                                    </p>
                                                }
                                                    .into_any()
                                            } else {
                                                view! {
                                                    <div class="paint-chips">
                                                        {color_list
                                                            .into_iter()
                                                            .map(|c| {
                                                                let id = c.id.clone();
                                                                let id2 = c.id.clone();
                                                                view! {
                                                                    <button
                                                                        class="paint-chip"
                                                                        style=format!("background-color: {}", c.hex)
                                                                        title=id.clone()
                                                                        on:click=move |_| add_paint(id2.clone())
                                                                    >
                                                                        <span>{c.id}</span>
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

                <div class="selected-paints">
                    <h2>"Your Mix"</h2>
                    {move || {
                        let paints = selected_paints.get();
                        if paints.is_empty() {
                            view! { <p class="hint">"Click paints above to add them to your mix"</p> }
                                .into_any()
                        } else {
                            view! {
                                <div class="mix-items">
                                    {paints
                                        .into_iter()
                                        .map(|(paint, weight)| {
                                            let paint_for_remove = paint.clone();
                                            let paint_for_update = paint.clone();
                                            view! {
                                                <div class="mix-item">
                                                    <span class="paint-name">{paint.clone()}</span>
                                                    <input
                                                        type="range"
                                                        min="0.1"
                                                        max="5"
                                                        step="0.1"
                                                        prop:value=weight.to_string()
                                                        on:input=move |ev| {
                                                            if let Ok(w) = event_target_value(&ev).parse::<f64>() {
                                                                update_weight(paint_for_update.clone(), w);
                                                            }
                                                        }
                                                    />
                                                    <span class="weight-value">{format!("{:.1}", weight)}</span>
                                                    <button
                                                        class="remove-btn"
                                                        on:click=move |_| remove_paint(paint_for_remove.clone())
                                                    >
                                                        "x"
                                                    </button>
                                                </div>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                                .into_any()
                        }
                    }}

                    <button
                        class="btn primary"
                        on:click=move |_| { calculate_mix.dispatch(()); }
                        disabled=move || loading.get() || selected_paints.get().len() < 2
                    >
                        {move || if loading.get() { "Calculating..." } else { "Calculate Mix" }}
                    </button>
                </div>

                <div class="mix-result">
                    <h2>"Result"</h2>
                    {move || {
                        error
                            .get()
                            .map(|e| {
                                view! { <div class="error-message">{e}</div> }
                            })
                    }}
                    {move || {
                        result_color
                            .get()
                            .map(|hex| {
                                view! {
                                    <div class="result-preview">
                                        <div class="result-swatch" style=format!("background-color: {}", hex)>
                                        </div>
                                        <span class="result-hex">{hex}</span>
                                    </div>
                                }
                            })
                    }}
                </div>
            </div>
        </div>
    }
}
