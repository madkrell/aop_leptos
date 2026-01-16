use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::Nav;
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/aop.css"/>
        <Title text="Artist Oil Paints - Spectral Colour Mixing"/>
        <Meta name="description" content="Mix accurate oil paint colours using spectral colour science"/>

        <Router>
            <Nav/>
            <main>
                <Routes fallback=|| view! { <h1>"404 - Page Not Found"</h1> }>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/login") view=LoginPage/>
                    <Route path=path!("/register") view=RegisterPage/>
                    <Route path=path!("/verify-email") view=VerifyEmailPage/>
                    <Route path=path!("/forgot-password") view=ForgotPasswordPage/>
                    <Route path=path!("/reset-password") view=ResetPasswordPage/>
                    <Route path=path!("/settings") view=SettingsPage/>
                    <Route path=path!("/target-mix") view=TargetMixPage/>
                    <Route path=path!("/test-mix") view=TestMixPage/>
                </Routes>
            </main>
        </Router>
    }
}
