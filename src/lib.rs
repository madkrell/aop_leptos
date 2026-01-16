pub mod app;
pub mod components;
pub mod models;
pub mod pages;
pub mod server_fns;

#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod services;
#[cfg(feature = "ssr")]
pub mod state;

// Re-export for convenience
pub use app::App;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
