use directories::ProjectDirs;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::{Arc, OnceLock};

static POOL: OnceLock<Arc<SqlitePool>> = OnceLock::new();

/// Initialize the database connection pool and schema.
pub async fn init() -> Result<(), sqlx::Error> {
    let proj_dirs = ProjectDirs::from("com", "MailDesk", "MailDesk")
        .expect("Failed to determine project directories");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
    let db_path = data_dir.join("maildesk.db");

    let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Create tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password TEXT,
            imap_server TEXT NOT NULL,
            imap_port INTEGER NOT NULL,
            smtp_server TEXT NOT NULL,
            smtp_port INTEGER NOT NULL,
            auth_type TEXT NOT NULL DEFAULT 'basic',
            access_token TEXT,
            refresh_token TEXT,
            token_expires_at INTEGER
        )",
    )
    .execute(&pool)
    .await?;

    // Store pool globally
    POOL.set(Arc::new(pool))
        .expect("Database pool already initialized");

    Ok(())
}

/// Get a reference to the database pool.
pub fn pool() -> Arc<SqlitePool> {
    POOL.get()
        .expect("Database not initialized. Call db::init() first.")
        .clone()
}
