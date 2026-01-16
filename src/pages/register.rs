use leptos::prelude::*;
use leptos_router::components::A;

use crate::server_fns::Register;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_action = ServerAction::<Register>::new();

    view! {
        <div class="auth-page">
            <div class="auth-card">
                <h1>"Create Account"</h1>

                {move || {
                    if let Some(Ok(_)) = register_action.value().get() {
                        return view! {
                            <div class="success-message">
                                <h2>"Check your email!"</h2>
                                <p>"We've sent a verification link to your email address."</p>
                                <p>"Please click the link to verify your account before signing in."</p>
                                <A href="/login" attr:class="btn btn-primary">"Go to Sign In"</A>
                            </div>
                        }.into_any();
                    }

                    view! {
                        <ActionForm action=register_action>
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
                                    minlength="8"
                                    placeholder="Minimum 8 characters"
                                />
                            </div>

                            <button type="submit" class="btn btn-primary" disabled=move || register_action.pending().get()>
                                {move || if register_action.pending().get() { "Creating account..." } else { "Create Account" }}
                            </button>

                            {move || register_action.value().get().map(|result| {
                                match result {
                                    Ok(_) => view! { <p class="success"></p> }.into_any(),
                                    Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                }
                            })}
                        </ActionForm>
                    }.into_any()
                }}

                <div class="auth-links">
                    <span>"Already have an account? "</span>
                    <A href="/login">"Sign In"</A>
                </div>
            </div>
        </div>
    }
}
