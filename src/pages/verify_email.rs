use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;

use crate::server_fns::verify_email;

#[component]
pub fn VerifyEmailPage() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.read().get("token").unwrap_or_default();

    let verify_result = Resource::new(token, |t| async move {
        if t.is_empty() {
            return Err("No verification token provided".to_string());
        }
        verify_email(t).await.map_err(|e| e.to_string())
    });

    view! {
        <div class="auth-page">
            <div class="auth-card">
                <h1>"Email Verification"</h1>

                <Suspense fallback=|| view! { <p>"Verifying your email..."</p> }>
                    {move || {
                        verify_result.get().map(|result| {
                            match result {
                                Ok(_) => view! {
                                    <div class="success-message">
                                        <h2>"Email Verified!"</h2>
                                        <p>"Your email has been verified successfully."</p>
                                        <A href="/login" attr:class="btn btn-primary">"Sign In"</A>
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="error-message">
                                        <h2>"Verification Failed"</h2>
                                        <p>{e}</p>
                                        <p>"The link may have expired or already been used."</p>
                                        <A href="/register" attr:class="btn btn-secondary">"Register Again"</A>
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
