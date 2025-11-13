use directories::ProjectDirs;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::{Arc, OnceLock};

static POOL: OnceLock<Arc<SqlitePool>> = OnceLock::new();

/// Initialize the database connection pool and schema.
pub async fn init() -> Result<(), sqlx::Error> {
    let proj_dirs = ProjectDirs::from("com", "Colimail", "Colimail")
        .expect("Failed to determine project directories");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
    let db_path = data_dir.join("colimail.db");

    println!("Database path: {}", db_path.display());

    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_str().unwrap());

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
            imap_server TEXT NOT NULL,
            imap_port INTEGER NOT NULL,
            smtp_server TEXT NOT NULL,
            smtp_port INTEGER NOT NULL,
            auth_type TEXT NOT NULL DEFAULT 'basic',
            display_name TEXT
        )",
    )
    .execute(&pool)
    .await?;

    // Migration: Add display_name column to existing accounts table if it doesn't exist
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN display_name TEXT")
        .execute(&pool)
        .await;

    // Create folders table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY,
            account_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            display_name TEXT NOT NULL,
            delimiter TEXT,
            flags TEXT,
            is_local INTEGER DEFAULT 0,
            UNIQUE(account_id, name),
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await?;

    // Migration: Add display_name column to existing folders table if it doesn't exist
    // This is safe because SQLite ignores ADD COLUMN if the column already exists
    let _ = sqlx::query("ALTER TABLE folders ADD COLUMN display_name TEXT DEFAULT ''")
        .execute(&pool)
        .await;

    // If display_name was just added and is empty, populate it from name
    // (for existing folders that were created before this migration)
    sqlx::query(
        "UPDATE folders SET display_name = name WHERE display_name = '' OR display_name IS NULL",
    )
    .execute(&pool)
    .await?;

    // Migration: Add is_local column to existing folders table if it doesn't exist
    let _ = sqlx::query("ALTER TABLE folders ADD COLUMN is_local INTEGER DEFAULT 0")
        .execute(&pool)
        .await;

    // Create emails cache table with all columns included
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS emails (
            id INTEGER PRIMARY KEY,
            account_id INTEGER NOT NULL,
            folder_name TEXT NOT NULL,
            uid INTEGER NOT NULL,
            subject TEXT NOT NULL,
            from_addr TEXT NOT NULL,
            to_addr TEXT NOT NULL,
            cc_addr TEXT,
            date TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            body TEXT,
            raw_headers TEXT,
            has_attachments INTEGER DEFAULT 0,
            flags TEXT,
            seen INTEGER DEFAULT 0,
            synced_at INTEGER NOT NULL,
            UNIQUE(account_id, folder_name, uid),
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await?;

    // Migration: Add cc_addr column to emails table for CC recipients (for existing tables)
    let _ = sqlx::query("ALTER TABLE emails ADD COLUMN cc_addr TEXT")
        .execute(&pool)
        .await;

    // Migration: Add has_attachments column to emails table if it doesn't exist (for existing tables)
    let _ = sqlx::query("ALTER TABLE emails ADD COLUMN has_attachments INTEGER DEFAULT 0")
        .execute(&pool)
        .await;

    // Migration: Add flags column to emails table for IMAP flags (for existing tables)
    let _ = sqlx::query("ALTER TABLE emails ADD COLUMN flags TEXT")
        .execute(&pool)
        .await;

    // Migration: Add seen column to emails table for read/unread status (for existing tables)
    let _ = sqlx::query("ALTER TABLE emails ADD COLUMN seen INTEGER DEFAULT 0")
        .execute(&pool)
        .await;

    // Migration: Add flagged column to emails table for starred/flagged status (for existing tables)
    let _ = sqlx::query("ALTER TABLE emails ADD COLUMN flagged INTEGER DEFAULT 0")
        .execute(&pool)
        .await;

    // Migration: Add synced_at column to emails table (for existing tables)
    let _ = sqlx::query("ALTER TABLE emails ADD COLUMN synced_at INTEGER")
        .execute(&pool)
        .await;

    // Migration: Add raw_headers column to emails table for CMVH verification caching (for existing tables)
    let _ = sqlx::query("ALTER TABLE emails ADD COLUMN raw_headers TEXT")
        .execute(&pool)
        .await;

    // Create index for faster queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_emails_account_folder
        ON emails(account_id, folder_name, timestamp DESC)",
    )
    .execute(&pool)
    .await?;

    // Create sync_status table to track last sync times and incremental sync state
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sync_status (
            id INTEGER PRIMARY KEY,
            account_id INTEGER NOT NULL,
            folder_name TEXT NOT NULL,
            last_sync_time INTEGER NOT NULL,
            uidvalidity INTEGER,
            highest_uid INTEGER,
            UNIQUE(account_id, folder_name),
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await?;

    // Migration: Add uidvalidity and highest_uid columns if they don't exist
    let _ = sqlx::query("ALTER TABLE sync_status ADD COLUMN uidvalidity INTEGER")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE sync_status ADD COLUMN highest_uid INTEGER")
        .execute(&pool)
        .await;

    // Create settings table for user preferences
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    // Set default sync interval if not exists
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('sync_interval', '300')")
        .execute(&pool)
        .await?;

    // Set default notification settings if not exists
    sqlx::query(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('notification_enabled', 'true')",
    )
    .execute(&pool)
    .await?;
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('sound_enabled', 'true')")
        .execute(&pool)
        .await?;

    // Set default minimize to tray setting if not exists
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('minimize_to_tray', 'true')")
        .execute(&pool)
        .await?;

    // Encryption settings
    sqlx::query(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('encryption_enabled', 'false')",
    )
    .execute(&pool)
    .await?;
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('encryption_salt', '')")
        .execute(&pool)
        .await?;
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('password_hash', '')")
        .execute(&pool)
        .await?;

    // Create attachments table for storing email attachments
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS attachments (
            id INTEGER PRIMARY KEY,
            email_id INTEGER NOT NULL,
            filename TEXT NOT NULL,
            content_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            data BLOB NOT NULL,
            FOREIGN KEY(email_id) REFERENCES emails(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await?;

    // Create index for faster attachment queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_attachments_email_id
        ON attachments(email_id)",
    )
    .execute(&pool)
    .await?;

    // Create drafts table for storing email drafts locally
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS drafts (
            id INTEGER PRIMARY KEY,
            account_id INTEGER NOT NULL,
            to_addr TEXT NOT NULL,
            cc_addr TEXT,
            subject TEXT NOT NULL,
            body TEXT NOT NULL,
            attachments TEXT,
            draft_type TEXT NOT NULL DEFAULT 'compose',
            original_email_id INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await?;

    // Create index for faster draft queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_drafts_account_updated
        ON drafts(account_id, updated_at DESC)",
    )
    .execute(&pool)
    .await?;

    // Create app_user table for storing authenticated user information
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS app_user (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            name TEXT,
            avatar_url TEXT,
            subscription_tier TEXT NOT NULL DEFAULT 'free',
            subscription_expires_at INTEGER,
            last_synced_at INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
    )
    .execute(&pool)
    .await?;

    // Migration: Add app_user_id to accounts table if it doesn't exist
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN app_user_id TEXT REFERENCES app_user(id)")
        .execute(&pool)
        .await;

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
