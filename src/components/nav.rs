use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos::web_sys;
use leptos_router::components::A;

use crate::server_fns::{get_current_user, Logout};

#[component]
pub fn Nav() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let logout_action = ServerAction::<Logout>::new();

    // After successful logout, do a full navigation to refresh the page state
    Effect::new(move |_| {
        if let Some(Ok(_)) = logout_action.value().get() {
            #[cfg(feature = "hydrate")]
            {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/");
                }
            }
        }
    });

    view! {
        <nav class="main-nav">
            <div class="nav-brand">
                <A href="/">"Artist Oil Paints"</A>
            </div>

            <div class="nav-links">
                <Suspense fallback=|| ()>
                    {move || {
                        user.get().map(|result| {
                            match result {
                                Ok(Some(u)) => view! {
                                    <A href="/target-mix">"Mix Colour"</A>
                                    <A href="/test-mix">"Test Mix"</A>
                                    <A href="/settings">"Settings"</A>
                                    <span class="user-email">{u.email}</span>
                                    <ActionForm action=logout_action attr:class="logout-form">
                                        <button type="submit" class="btn btn-small">"Sign Out"</button>
                                    </ActionForm>
                                }.into_any(),
                                _ => view! {
                                    <A href="/login">"Sign In"</A>
                                    <A href="/register">"Register"</A>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </nav>
    }
}
