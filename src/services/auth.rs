use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::db::{self, Db, User};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Email already registered")]
    EmailExists,
    #[error("Account locked")]
    AccountLocked,
    #[error("Invalid or expired token")]
    InvalidToken,
    #[error("Email not verified")]
    EmailNotVerified,
    #[error("{0}")]
    Other(String),
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AuthError::Other(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub async fn register(db: &Db, email: &str, password: &str) -> Result<String, AuthError> {
    if db::get_user_by_email(db, email).await.is_some() {
        return Err(AuthError::EmailExists);
    }
    let id = Uuid::new_v4().to_string();
    let hash = hash_password(password)?;
    db::create_user(db, &id, email, &hash)
        .await
        .map_err(|e| AuthError::Other(e.to_string()))?;
    Ok(id)
}

pub async fn login(db: &Db, email: &str, password: &str) -> Result<User, AuthError> {
    let user = db::get_user_by_email(db, email)
        .await
        .ok_or(AuthError::InvalidCredentials)?;

    // Check lockout
    if let Some(ref locked) = user.locked_until {
        if chrono::DateTime::parse_from_rfc3339(locked)
            .map(|t| t > Utc::now())
            .unwrap_or(false)
        {
            return Err(AuthError::AccountLocked);
        }
    }

    if !verify_password(password, &user.password_hash) {
        let attempts = user.failed_attempts + 1;
        let locked = if attempts >= 5 {
            Some((Utc::now() + Duration::minutes(15)).to_rfc3339())
        } else {
            None
        };
        let _ = db::update_failed_attempts(db, &user.id, attempts, locked.as_deref()).await;
        return Err(AuthError::InvalidCredentials);
    }

    let _ = db::update_failed_attempts(db, &user.id, 0, None).await;
    Ok(user)
}

pub async fn create_verification_token(db: &Db, user_id: &str) -> Result<String, AuthError> {
    create_token(db, user_id, "verify", 24).await
}

pub async fn create_reset_token(db: &Db, user_id: &str) -> Result<String, AuthError> {
    create_token(db, user_id, "reset", 1).await
}

async fn create_token(db: &Db, user_id: &str, kind: &str, hours: i64) -> Result<String, AuthError> {
    let token = Uuid::new_v4().to_string();
    let expires = (Utc::now() + Duration::hours(hours)).to_rfc3339();
    db::create_token(
        db,
        &Uuid::new_v4().to_string(),
        user_id,
        kind,
        &hash_token(&token),
        &expires,
    )
    .await
    .map_err(|e| AuthError::Other(e.to_string()))?;
    Ok(token)
}

pub async fn verify_token(db: &Db, token: &str, kind: &str) -> Result<String, AuthError> {
    let (id, user_id, expires) = db::get_token(db, &hash_token(token), kind)
        .await
        .ok_or(AuthError::InvalidToken)?;
    if chrono::DateTime::parse_from_rfc3339(&expires)
        .map(|t| t < Utc::now())
        .unwrap_or(true)
    {
        return Err(AuthError::InvalidToken);
    }
    let _ = db::delete_token(db, &id).await;
    Ok(user_id)
}

pub async fn verify_email(db: &Db, token: &str) -> Result<(), AuthError> {
    let user_id = verify_token(db, token, "verify").await?;
    db::verify_user_email(db, &user_id)
        .await
        .map_err(|e| AuthError::Other(e.to_string()))
}

pub async fn reset_password(db: &Db, token: &str, new_password: &str) -> Result<(), AuthError> {
    let user_id = verify_token(db, token, "reset").await?;
    let hash = hash_password(new_password)?;
    db::update_password(db, &user_id, &hash)
        .await
        .map_err(|e| AuthError::Other(e.to_string()))
}
