use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="home-page">
            <section class="hero">
                <div class="hero-content">
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
                </div>
                <div class="waves">
                    <svg class="waves-svg" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
                        viewBox="0 24 150 28" preserveAspectRatio="none" shape-rendering="auto">
                        <defs>
                            <path id="wave-path" d="M-160 44c30 0 58-18 88-18s 58 18 88 18 58-18 88-18 58 18 88 18 v44h-352z" />
                        </defs>
                        <g class="wave-parallax">
                            <use xlink:href="#wave-path" x="48" y="0" class="wave wave-1" />
                            <use xlink:href="#wave-path" x="48" y="3" class="wave wave-2" />
                            <use xlink:href="#wave-path" x="48" y="5" class="wave wave-3" />
                            <use xlink:href="#wave-path" x="48" y="7" class="wave wave-4" />
                        </g>
                    </svg>
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

            <section class="footer-wave">
                <div class="waves waves-inverted">
                    <svg class="waves-svg" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
                        viewBox="0 24 150 28" preserveAspectRatio="none" shape-rendering="auto">
                        <defs>
                            <path id="wave-path-footer" d="M-160 44c30 0 58-18 88-18s 58 18 88 18 58-18 88-18 58 18 88 18 v44h-352z" />
                        </defs>
                        <g class="wave-parallax">
                            <use xlink:href="#wave-path-footer" x="48" y="0" class="wave wave-1" />
                            <use xlink:href="#wave-path-footer" x="48" y="3" class="wave wave-2" />
                            <use xlink:href="#wave-path-footer" x="48" y="5" class="wave wave-3" />
                            <use xlink:href="#wave-path-footer" x="48" y="7" class="wave wave-4" />
                        </g>
                    </svg>
                </div>
                <div class="footer-content">
                </div>
            </section>
        </div>
    }
}
