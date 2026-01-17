#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use aop::{state::AppState, App};
    use axum::{Extension, Router};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::sync::Arc;
    use tower_sessions::{MemoryStore, SessionManagerLayer};

    // Load env vars
    dotenvy::dotenv().ok();

    // Initialize database
    // Check multiple locations: env var, current dir, then target/site (oxyde.cloud workaround)
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        if std::path::Path::new("data.db").exists() {
            "sqlite:data.db".into()
        } else if std::path::Path::new("target/site/data.db").exists() {
            "sqlite:target/site/data.db".into()
        } else {
            "sqlite:data.db".into()
        }
    });
    let db = aop::db::create_pool(&db_url).await;

    // Run migrations
    aop::db::run_migrations(&db).await;

    // Create app state
    let state = AppState {
        db: db.clone(),
        email: Arc::new(aop::services::email::Email {
            api_key: std::env::var("RESEND_API_KEY").unwrap_or_default(),
            from: std::env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "noreply@artistoilpaints.co.uk".into()),
            base_url: std::env::var("BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".into()),
        }),
    };

    // Session store
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(std::env::var("PRODUCTION").is_ok())
        .with_same_site(tower_sessions::cookie::SameSite::Lax);

    // Leptos config - try Cargo.toml first, fall back to environment/defaults for packaged app
    let leptos_options = match get_configuration(Some("Cargo.toml")) {
        Ok(conf) => conf.leptos_options,
        Err(_) => {
            // Running as packaged app - build config from env vars
            use leptos::config::LeptosOptions;
            use std::net::SocketAddr;

            let site_root = std::env::var("LEPTOS_SITE_ROOT").unwrap_or_else(|_| "site".into());
            let site_addr: SocketAddr = std::env::var("LEPTOS_SITE_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:3000".into())
                .parse()
                .expect("Invalid LEPTOS_SITE_ADDR");

            LeptosOptions::builder()
                .output_name("aop")
                .site_root(site_root)
                .site_pkg_dir("pkg")
                .site_addr(site_addr)
                .reload_port(3001)
                .build()
        }
    };
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Build router
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(Extension(state))
        .layer(session_layer)
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://{}", addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(feature = "ssr")]
fn shell(options: leptos::config::LeptosOptions) -> impl leptos::IntoView {
    use aop::App;
    use leptos::prelude::*;
    use leptos_meta::*;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // Client-side entry point handled by hydrate() in lib.rs
}
