use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="home-page">
            <section class="hero">
                <h1>"Artist Oil Paints"</h1>
                <p class="subtitle">"Spectral Colour Mixing for Accurate Paint Colours"</p>
                <p class="description">
                    "Mix oil paint colours with scientific precision using spectral colour science. "
                    "Get accurate colour predictions based on real paint pigment data."
                </p>
                <div class="cta-buttons">
                    <A href="/login" attr:class="btn btn-primary">"Get Started"</A>
                    <A href="/register" attr:class="btn btn-secondary">"Create Account"</A>
                </div>
            </section>

            <section class="features">
                <div class="feature">
                    <h3>"Spectral Mixing"</h3>
                    <p>"True spectral colour mixing that accounts for subtractive pigment behaviour"</p>
                </div>
                <div class="feature">
                    <h3>"Real Paint Data"</h3>
                    <p>"Based on measured spectral data from actual oil paint brands"</p>
                </div>
                <div class="feature">
                    <h3>"Optimised Recipes"</h3>
                    <p>"Get optimal paint mixing ratios to match any target colour"</p>
                </div>
            </section>
        </div>
    }
}
