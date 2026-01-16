use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SessionUser {
    pub id: String,
    pub email: String,
}

#[server]
pub async fn get_current_user() -> Result<Option<SessionUser>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use tower_sessions::Session;

    let Extension(session) = extract::<Extension<Session>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(session.get("user").await.ok().flatten())
}

#[server]
pub async fn login(email: String, password: String) -> Result<SessionUser, ServerFnError> {
    use axum::Extension;
    use crate::{services::auth, state::AppState};
    use leptos_axum::extract;
    use tower_sessions::Session;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let Extension(session) = extract::<Extension<Session>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user = auth::login(&state.db, &email, &password)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !user.email_verified {
        return Err(ServerFnError::new("Please verify your email first"));
    }

    let session_user = SessionUser {
        id: user.id,
        email: user.email,
    };
    session.insert("user", &session_user).await?;
    Ok(session_user)
}

#[server]
pub async fn register(email: String, password: String) -> Result<(), ServerFnError> {
    use axum::Extension;
    use crate::{services::auth, state::AppState};
    use leptos_axum::extract;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_id = auth::register(&state.db, &email, &password)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let token = auth::create_verification_token(&state.db, &user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    state
        .email
        .send_verification(&email, &token)
        .await
        .map_err(ServerFnError::new)?;

    Ok(())
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use tower_sessions::Session;

    let Extension(session) = extract::<Extension<Session>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    session.delete().await?;
    Ok(())
}

#[server]
pub async fn verify_email(token: String) -> Result<(), ServerFnError> {
    use axum::Extension;
    use crate::{services::auth, state::AppState};
    use leptos_axum::extract;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    auth::verify_email(&state.db, &token)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn request_password_reset(email: String) -> Result<(), ServerFnError> {
    use axum::Extension;
    use crate::{db, services::auth, state::AppState};
    use leptos_axum::extract;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Always succeed to prevent email enumeration
    if let Some(user) = db::get_user_by_email(&state.db, &email).await {
        if let Ok(token) = auth::create_reset_token(&state.db, &user.id).await {
            let _ = state.email.send_password_reset(&email, &token).await;
        }
    }

    Ok(())
}

#[server]
pub async fn reset_password(token: String, password: String) -> Result<(), ServerFnError> {
    use axum::Extension;
    use crate::{services::auth, state::AppState};
    use leptos_axum::extract;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    auth::reset_password(&state.db, &token, &password)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
