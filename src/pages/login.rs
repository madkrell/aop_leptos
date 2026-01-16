use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos::web_sys;
use leptos_router::components::A;

use crate::server_fns::Login;

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = ServerAction::<Login>::new();

    // After successful login, do a full navigation to refresh the page state
    Effect::new(move |_| {
        if let Some(Ok(_)) = login_action.value().get() {
            // Use window.location for a full page navigation to ensure session is picked up
            #[cfg(feature = "hydrate")]
            {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/");
                }
            }
        }
    });

    view! {
        <div class="auth-page">
            <div class="auth-card">
                <h1>"Sign In"</h1>

                <ActionForm action=login_action>
                    <div class="form-group">
                        <label for="email">"Email"</label>
                        <input
                            type="email"
                            id="email"
                            name="email"
                            required
                            placeholder="your@email.com"
                        />
                    </div>

                    <div class="form-group">
                        <label for="password">"Password"</label>
                        <input
                            type="password"
                            id="password"
                            name="password"
                            required
                            placeholder="••••••••"
                        />
                    </div>

                    <button type="submit" class="btn btn-primary" disabled=move || login_action.pending().get()>
                        {move || if login_action.pending().get() { "Signing in..." } else { "Sign In" }}
                    </button>

                    {move || login_action.value().get().map(|result| {
                        match result {
                            Ok(_) => view! { <p class="success">"Login successful! Redirecting..."</p> }.into_any(),
                            Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                        }
                    })}
                </ActionForm>

                <div class="auth-links">
                    <A href="/forgot-password">"Forgot password?"</A>
                    <span>" | "</span>
                    <A href="/register">"Create account"</A>
                </div>
            </div>
        </div>
    }
}
