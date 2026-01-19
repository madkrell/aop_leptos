use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

pub type Db = Pool<Sqlite>;

// User model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub email_verified: bool,
    pub password_hash: String,
    pub created_at: String,
    pub failed_attempts: i32,
    pub locked_until: Option<String>,
}

// Create connection pool - tries multiple paths for database file
pub async fn create_pool(url: &str) -> Db {
    // List of paths to try (in order of preference)
    let paths_to_try = [
        url.to_string(),
        "sqlite:data.db".to_string(),
        "sqlite:./data.db".to_string(),
        "sqlite:target/site/data.db".to_string(),
    ];

    for db_url in &paths_to_try {
        // Extract path from sqlite: URL
        if let Some(path) = db_url.strip_prefix("sqlite:") {
            let path = path.trim_start_matches("./");
            if std::path::Path::new(path).exists() {
                println!("Found database at: {}", path);
                return sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(20)
                    .connect(db_url)
                    .await
                    .expect("Failed to connect to database");
            }
        }
    }

    // If no file found, print debug info and panic
    eprintln!("ERROR: Could not find database file!");
    eprintln!("Tried paths: {:?}", paths_to_try);
    if let Ok(cwd) = std::env::current_dir() {
        eprintln!("Current working directory: {:?}", cwd);
    }
    if let Ok(entries) = std::fs::read_dir(".") {
        eprintln!("Files in current directory:");
        for entry in entries.flatten() {
            eprintln!("  {:?}", entry.path());
        }
    }
    panic!("Database file not found");
}

// Run migrations (create tables if not exist)
pub async fn run_migrations(db: &Db) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            email_verified INTEGER DEFAULT 0,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            failed_attempts INTEGER DEFAULT 0,
            locked_until TEXT
        )
        "#,
    )
    .execute(db)
    .await
    .expect("Failed to create users table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tokens (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            kind TEXT NOT NULL,
            hash TEXT NOT NULL,
            expires_at TEXT NOT NULL
        )
        "#,
    )
    .execute(db)
    .await
    .expect("Failed to create tokens table");

    // Migrate user_settings table to have proper primary key
    // Check if old table exists without primary key
    let has_pk: Option<(i32,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('user_settings') WHERE pk = 1"
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    if has_pk.map(|(c,)| c).unwrap_or(0) == 0 {
        // Table exists but without primary key - migrate it
        let _ = sqlx::query("ALTER TABLE user_settings RENAME TO user_settings_old")
            .execute(db)
            .await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_settings (
                _id TEXT PRIMARY KEY,
                email TEXT,
                colour_mix_choice TEXT,
                selected_colors TEXT
            )
            "#,
        )
        .execute(db)
        .await
        .expect("Failed to create user_settings table");

        // Copy data from old table
        let _ = sqlx::query(
            r#"
            INSERT OR REPLACE INTO user_settings (_id, email, colour_mix_choice, selected_colors)
            SELECT _id, email, colour_mix_choice, selected_colors FROM user_settings_old
            WHERE _id IS NOT NULL
            "#,
        )
        .execute(db)
        .await;

        let _ = sqlx::query("DROP TABLE IF EXISTS user_settings_old")
            .execute(db)
            .await;
    } else {
        // Just create if doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_settings (
                _id TEXT PRIMARY KEY,
                email TEXT,
                colour_mix_choice TEXT,
                selected_colors TEXT
            )
            "#,
        )
        .execute(db)
        .await
        .expect("Failed to create user_settings table");
    }
}

// User queries
pub async fn get_user_by_email(db: &Db, email: &str) -> Option<User> {
    sqlx::query_as("SELECT * FROM users WHERE email = ?")
        .bind(email.to_lowercase())
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
}

pub async fn get_user_by_id(db: &Db, id: &str) -> Option<User> {
    sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
}

pub async fn create_user(
    db: &Db,
    id: &str,
    email: &str,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, datetime('now'))",
    )
    .bind(id)
    .bind(email.to_lowercase())
    .bind(password_hash)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn verify_user_email(db: &Db, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET email_verified = 1 WHERE id = ?")
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn update_password(db: &Db, user_id: &str, hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(hash)
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn update_failed_attempts(
    db: &Db,
    user_id: &str,
    count: i32,
    locked_until: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET failed_attempts = ?, locked_until = ? WHERE id = ?")
        .bind(count)
        .bind(locked_until)
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(())
}

// Token queries
pub async fn create_token(
    db: &Db,
    id: &str,
    user_id: &str,
    kind: &str,
    hash: &str,
    expires_at: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO tokens (id, user_id, kind, hash, expires_at) VALUES (?, ?, ?, ?, ?)")
        .bind(id)
        .bind(user_id)
        .bind(kind)
        .bind(hash)
        .bind(expires_at)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn get_token(db: &Db, hash: &str, kind: &str) -> Option<(String, String, String)> {
    sqlx::query_as("SELECT id, user_id, expires_at FROM tokens WHERE hash = ? AND kind = ?")
        .bind(hash)
        .bind(kind)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
}

pub async fn delete_token(db: &Db, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM tokens WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

// User settings queries
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSettings {
    pub _id: String,
    pub email: Option<String>,
    pub colour_mix_choice: Option<String>,
    pub selected_colors: Option<String>,
}

pub async fn get_user_settings(db: &Db, user_id: &str) -> Option<UserSettings> {
    sqlx::query_as("SELECT * FROM user_settings WHERE _id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
}

pub async fn upsert_user_settings(
    db: &Db,
    user_id: &str,
    email: &str,
    mix_choice: &str,
    selected_colors: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_settings (_id, email, colour_mix_choice, selected_colors)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(_id) DO UPDATE SET
            email = excluded.email,
            colour_mix_choice = excluded.colour_mix_choice,
            selected_colors = excluded.selected_colors
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind(mix_choice)
    .bind(selected_colors)
    .execute(db)
    .await?;
    Ok(())
}

// Paint data queries
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaintColor {
    pub _id: String,
    pub spectral_curve: Option<Vec<u8>>,
    pub d65_10deg_hex: Option<String>,
}

pub async fn get_paint_brands(_db: &Db) -> Vec<String> {
    // Return list of paint brand table names
    vec![
        "winsor_newton_artist_oil_colour".into(),
        "daler_rowney_georgian_oil_colours".into(),
        "griffin_alkyd_fast_drying_oil_colour".into(),
        "gamblin_conservation_colors".into(),
        "michael_harding".into(),
        "maimeri_puro_oil".into(),
        "schmincke_mussini_oils".into(),
        "sennellier_extra_fine_oils".into(),
        "talens_van_gogh_oil_colour".into(),
        "williamsburg_handmade_oil_colors".into(),
        "winton_oil_colour".into(),
    ]
}

pub async fn get_paint_colors(db: &Db, brand: &str) -> Vec<PaintColor> {
    // Sanitize brand name to prevent SQL injection
    let valid_brands = get_paint_brands(db).await;
    if !valid_brands.contains(&brand.to_string()) {
        return vec![];
    }

    let query = format!("SELECT _id, spectral_curve, d65_10deg_hex FROM {}", brand);
    sqlx::query_as(&query)
        .fetch_all(db)
        .await
        .unwrap_or_default()
}

pub async fn get_spectral_data(db: &Db, brand: &str, color: &str) -> Option<Vec<u8>> {
    let valid_brands = get_paint_brands(db).await;
    if !valid_brands.contains(&brand.to_string()) {
        return None;
    }

    let query = format!("SELECT spectral_curve FROM {} WHERE _id = ?", brand);
    let result: Option<(Vec<u8>,)> = sqlx::query_as(&query)
        .bind(color)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    result.map(|(data,)| data)
}
