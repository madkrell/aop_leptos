use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;

use crate::server_fns::ResetPassword;

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.read().get("token").unwrap_or_default();
    let reset_action = ServerAction::<ResetPassword>::new();

    view! {
        <div class="auth-page">
            <div class="auth-card">
                <h1>"Set New Password"</h1>

                {move || {
                    let t = token();
                    if t.is_empty() {
                        return view! {
                            <div class="error-message">
                                <p>"Invalid reset link. Please request a new password reset."</p>
                                <A href="/forgot-password" attr:class="btn btn-secondary">"Request Reset"</A>
                            </div>
                        }.into_any();
                    }

                    if let Some(Ok(_)) = reset_action.value().get() {
                        return view! {
                            <div class="success-message">
                                <h2>"Password Updated!"</h2>
                                <p>"Your password has been reset successfully."</p>
                                <A href="/login" attr:class="btn btn-primary">"Sign In"</A>
                            </div>
                        }.into_any();
                    }

                    view! {
                        <ActionForm action=reset_action>
                            <input type="hidden" name="token" value=t />

                            <div class="form-group">
                                <label for="password">"New Password"</label>
                                <input
                                    type="password"
                                    id="password"
                                    name="password"
                                    required
                                    minlength="8"
                                    placeholder="Minimum 8 characters"
                                />
                            </div>

                            <button type="submit" class="btn btn-primary" disabled=move || reset_action.pending().get()>
                                {move || if reset_action.pending().get() { "Updating..." } else { "Update Password" }}
                            </button>

                            {move || reset_action.value().get().map(|result| {
                                match result {
                                    Ok(_) => view! { <p class="success"></p> }.into_any(),
                                    Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                                }
                            })}
                        </ActionForm>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
