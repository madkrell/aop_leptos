use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::server_fns::get_current_user;

#[component]
pub fn AuthGuard(children: ChildrenFn) -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(None)) = user.get() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <Suspense fallback=|| view! { <div class="loading">"Loading..."</div> }>
            {move || {
                user.get().map(|result| {
                    match result {
                        Ok(Some(_)) => children().into_any(),
                        _ => view! { <div class="loading">"Redirecting..."</div> }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
}
