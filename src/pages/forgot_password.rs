use leptos::prelude::*;
use leptos_router::components::A;

use crate::server_fns::RequestPasswordReset;

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    let reset_action = ServerAction::<RequestPasswordReset>::new();

    view! {
        <div class="auth-page">
            <div class="auth-card">
                <h1>"Reset Password"</h1>

                {move || {
                    if let Some(Ok(_)) = reset_action.value().get() {
                        return view! {
                            <div class="success-message">
                                <h2>"Check your email"</h2>
                                <p>"If an account exists with that email, we've sent a password reset link."</p>
                                <p>"The link will expire in 1 hour."</p>
                                <A href="/login" attr:class="btn btn-secondary">"Back to Sign In"</A>
                            </div>
                        }.into_any();
                    }

                    view! {
                        <div>
                            <p class="instructions">"Enter your email and we'll send you a link to reset your password."</p>

                            <ActionForm action=reset_action>
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

                                <button type="submit" class="btn btn-primary" disabled=move || reset_action.pending().get()>
                                    {move || if reset_action.pending().get() { "Sending..." } else { "Send Reset Link" }}
                                </button>

                                {move || reset_action.value().get().map(|result| {
                                    match result {
                                        Ok(_) => view! { <p class="success"></p> }.into_any(),
                                        Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                    }
                                })}
                            </ActionForm>
                        </div>
                    }.into_any()
                }}

                <div class="auth-links">
                    <A href="/login">"Back to Sign In"</A>
                </div>
            </div>
        </div>
    }
}
