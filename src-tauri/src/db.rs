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

    let db_url = format!(
        "sqlite://{}?mode=rwc",
        db_path.to_str().ok_or_else(|| sqlx::Error::Configuration(
            "Database path contains invalid UTF-8".into()
        ))?
    );

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Enable WAL mode for better concurrency (allows readers during write operations)
    // This significantly improves responsiveness in desktop applications
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    // Set synchronous to NORMAL for better performance while maintaining durability
    // NORMAL is safe for most applications and much faster than FULL
    sqlx::query("PRAGMA synchronous=NORMAL")
        .execute(&pool)
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

    // Create CMVH verification cache table for on-chain verification results
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cmvh_verification_cache (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            signature TEXT NOT NULL,
            email_hash TEXT NOT NULL,
            is_valid INTEGER NOT NULL,
            error TEXT,
            verified_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            UNIQUE(signature, email_hash)
        )",
    )
    .execute(&pool)
    .await?;

    // Create indexes for CMVH cache queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_cmvh_signature
        ON cmvh_verification_cache(signature)",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_cmvh_expires
        ON cmvh_verification_cache(expires_at)",
    )
    .execute(&pool)
    .await?;

    // Create ENS name cache table for reverse resolution results
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ens_cache (
            address TEXT PRIMARY KEY,
            ens_name TEXT,
            resolved_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    // Create index for ENS cache expiration queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ens_expires
        ON ens_cache(expires_at)",
    )
    .execute(&pool)
    .await?;

    // Create reward cache table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS reward_cache (
            reward_id TEXT PRIMARY KEY,
            email_hash TEXT UNIQUE NOT NULL,
            amount TEXT NOT NULL,
            sender TEXT NOT NULL,
            recipient TEXT NOT NULL,
            status TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    // Create index for reward cache queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_reward_email_hash
        ON reward_cache(email_hash)",
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    /// Initialize an in-memory test database with the same schema as production
    async fn init_test_db() -> Result<SqlitePool, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;

        // Enable WAL mode for better concurrency
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await?;

        sqlx::query("PRAGMA synchronous=NORMAL")
            .execute(&pool)
            .await?;

        // Create all tables (same as init() but without POOL storage)
        // Accounts table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                imap_server TEXT NOT NULL,
                imap_port INTEGER NOT NULL,
                smtp_server TEXT NOT NULL,
                smtp_port INTEGER NOT NULL,
                auth_type TEXT NOT NULL DEFAULT 'basic',
                display_name TEXT,
                app_user_id TEXT
            )",
        )
        .execute(&pool)
        .await?;

        // Folders table
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

        // Emails table
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
                flagged INTEGER DEFAULT 0,
                synced_at INTEGER NOT NULL,
                UNIQUE(account_id, folder_name, uid),
                FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
            )",
        )
        .execute(&pool)
        .await?;

        // Settings table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        // ENS cache table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ens_cache (
                address TEXT PRIMARY KEY,
                ens_name TEXT,
                resolved_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        Ok(pool)
    }

    #[tokio::test]
    async fn test_account_crud_operations() {
        let pool = init_test_db().await.expect("Failed to init test database");

        // Test INSERT account
        let insert_result = sqlx::query(
            "INSERT INTO accounts (email, imap_server, imap_port, smtp_server, smtp_port, auth_type, display_name)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("test@example.com")
        .bind("imap.example.com")
        .bind(993)
        .bind("smtp.example.com")
        .bind(587)
        .bind("basic")
        .bind("Test User")
        .execute(&pool)
        .await;

        assert!(insert_result.is_ok(), "Failed to insert account");
        let account_id = insert_result.unwrap().last_insert_rowid();
        assert_eq!(account_id, 1, "First account should have ID 1");

        // Test SELECT account
        let row = sqlx::query("SELECT email, imap_server, imap_port, display_name FROM accounts WHERE id = ?")
            .bind(account_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch account");

        assert_eq!(row.get::<String, _>("email"), "test@example.com");
        assert_eq!(row.get::<String, _>("imap_server"), "imap.example.com");
        assert_eq!(row.get::<i64, _>("imap_port"), 993);
        assert_eq!(row.get::<String, _>("display_name"), "Test User");

        // Test UPDATE account
        let update_result = sqlx::query("UPDATE accounts SET display_name = ? WHERE id = ?")
            .bind("Updated User")
            .bind(account_id)
            .execute(&pool)
            .await;

        assert!(update_result.is_ok(), "Failed to update account");
        assert_eq!(update_result.unwrap().rows_affected(), 1, "Should update exactly 1 row");

        // Verify update
        let updated_row = sqlx::query("SELECT display_name FROM accounts WHERE id = ?")
            .bind(account_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch updated account");

        assert_eq!(updated_row.get::<String, _>("display_name"), "Updated User");

        // Test DELETE account
        let delete_result = sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(account_id)
            .execute(&pool)
            .await;

        assert!(delete_result.is_ok(), "Failed to delete account");
        assert_eq!(delete_result.unwrap().rows_affected(), 1, "Should delete exactly 1 row");

        // Verify deletion
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE id = ?")
            .bind(account_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count accounts");

        assert_eq!(count, 0, "Account should be deleted");
    }

    #[tokio::test]
    async fn test_unique_constraint_on_email() {
        let pool = init_test_db().await.expect("Failed to init test database");

        // Insert first account
        sqlx::query(
            "INSERT INTO accounts (email, imap_server, imap_port, smtp_server, smtp_port)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("duplicate@example.com")
        .bind("imap.example.com")
        .bind(993)
        .bind("smtp.example.com")
        .bind(587)
        .execute(&pool)
        .await
        .expect("First insert should succeed");

        // Try to insert duplicate email
        let duplicate_result = sqlx::query(
            "INSERT INTO accounts (email, imap_server, imap_port, smtp_server, smtp_port)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("duplicate@example.com")
        .bind("imap2.example.com")
        .bind(993)
        .bind("smtp2.example.com")
        .bind(587)
        .execute(&pool)
        .await;

        assert!(duplicate_result.is_err(), "Duplicate email should violate UNIQUE constraint");
    }

    #[tokio::test]
    async fn test_folder_foreign_key_cascade() {
        let pool = init_test_db().await.expect("Failed to init test database");

        // Insert account
        let account_id = sqlx::query(
            "INSERT INTO accounts (email, imap_server, imap_port, smtp_server, smtp_port)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("test@example.com")
        .bind("imap.example.com")
        .bind(993)
        .bind("smtp.example.com")
        .bind(587)
        .execute(&pool)
        .await
        .expect("Failed to insert account")
        .last_insert_rowid();

        // Insert folder
        sqlx::query(
            "INSERT INTO folders (account_id, name, display_name)
             VALUES (?, ?, ?)"
        )
        .bind(account_id)
        .bind("INBOX")
        .bind("Inbox")
        .execute(&pool)
        .await
        .expect("Failed to insert folder");

        // Verify folder exists
        let folder_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM folders WHERE account_id = ?")
            .bind(account_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count folders");

        assert_eq!(folder_count, 1, "Should have 1 folder");

        // Delete account (should cascade delete folder)
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(account_id)
            .execute(&pool)
            .await
            .expect("Failed to delete account");

        // Verify folder is also deleted (CASCADE)
        let folder_count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM folders WHERE account_id = ?")
            .bind(account_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count folders after delete");

        assert_eq!(folder_count_after, 0, "Folder should be cascade deleted");
    }

    #[tokio::test]
    async fn test_email_unique_constraint() {
        let pool = init_test_db().await.expect("Failed to init test database");

        // Insert account
        let account_id = sqlx::query(
            "INSERT INTO accounts (email, imap_server, imap_port, smtp_server, smtp_port)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("test@example.com")
        .bind("imap.example.com")
        .bind(993)
        .bind("smtp.example.com")
        .bind(587)
        .execute(&pool)
        .await
        .expect("Failed to insert account")
        .last_insert_rowid();

        let now = chrono::Utc::now().timestamp();

        // Insert first email
        sqlx::query(
            "INSERT INTO emails (account_id, folder_name, uid, subject, from_addr, to_addr, date, timestamp, synced_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(account_id)
        .bind("INBOX")
        .bind(12345)
        .bind("Test Subject")
        .bind("sender@example.com")
        .bind("receiver@example.com")
        .bind("Mon, 15 Jan 2024 14:30:00 +0000")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("First email insert should succeed");

        // Try to insert duplicate (same account_id, folder_name, uid)
        let duplicate_result = sqlx::query(
            "INSERT INTO emails (account_id, folder_name, uid, subject, from_addr, to_addr, date, timestamp, synced_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(account_id)
        .bind("INBOX")
        .bind(12345)  // Same UID
        .bind("Different Subject")
        .bind("sender2@example.com")
        .bind("receiver2@example.com")
        .bind("Mon, 16 Jan 2024 14:30:00 +0000")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        assert!(duplicate_result.is_err(), "Duplicate email (account_id, folder_name, uid) should violate UNIQUE constraint");
    }

    #[tokio::test]
    async fn test_settings_upsert() {
        let pool = init_test_db().await.expect("Failed to init test database");

        // Insert initial setting
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
            .bind("sync_interval")
            .bind("300")
            .execute(&pool)
            .await
            .expect("Failed to insert setting");

        // Verify initial value
        let initial_value: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
            .bind("sync_interval")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch setting");

        assert_eq!(initial_value, "300");

        // Update using INSERT OR REPLACE (upsert)
        sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
            .bind("sync_interval")
            .bind("600")
            .execute(&pool)
            .await
            .expect("Failed to upsert setting");

        // Verify updated value
        let updated_value: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
            .bind("sync_interval")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch updated setting");

        assert_eq!(updated_value, "600");

        // Verify only 1 row exists (not 2)
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings WHERE key = ?")
            .bind("sync_interval")
            .fetch_one(&pool)
            .await
            .expect("Failed to count settings");

        assert_eq!(count, 1, "Should have exactly 1 row after upsert");
    }

    #[tokio::test]
    async fn test_ens_cache_expiration_query() {
        let pool = init_test_db().await.expect("Failed to init test database");

        let now = chrono::Utc::now().timestamp();
        let one_day_ago = now - 86400;
        let one_day_later = now + 86400;

        // Insert expired entry
        sqlx::query(
            "INSERT INTO ens_cache (address, ens_name, resolved_at, expires_at)
             VALUES (?, ?, ?, ?)"
        )
        .bind("0x1111111111111111111111111111111111111111")
        .bind(Some("expired.eth"))
        .bind(one_day_ago)
        .bind(one_day_ago)  // Expired
        .execute(&pool)
        .await
        .expect("Failed to insert expired ENS cache");

        // Insert valid entry
        sqlx::query(
            "INSERT INTO ens_cache (address, ens_name, resolved_at, expires_at)
             VALUES (?, ?, ?, ?)"
        )
        .bind("0x2222222222222222222222222222222222222222")
        .bind(Some("valid.eth"))
        .bind(now)
        .bind(one_day_later)  // Not expired
        .execute(&pool)
        .await
        .expect("Failed to insert valid ENS cache");

        // Query only valid (not expired) entries
        let valid_entries: Vec<String> = sqlx::query_scalar(
            "SELECT address FROM ens_cache WHERE expires_at > ?"
        )
        .bind(now)
        .fetch_all(&pool)
        .await
        .expect("Failed to query valid ENS entries");

        assert_eq!(valid_entries.len(), 1, "Should have 1 valid entry");
        assert_eq!(valid_entries[0], "0x2222222222222222222222222222222222222222");

        // Test cleanup of expired entries
        let delete_result = sqlx::query("DELETE FROM ens_cache WHERE expires_at <= ?")
            .bind(now)
            .execute(&pool)
            .await
            .expect("Failed to delete expired entries");

        assert_eq!(delete_result.rows_affected(), 1, "Should delete 1 expired entry");

        // Verify only valid entry remains
        let remaining_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ens_cache")
            .fetch_one(&pool)
            .await
            .expect("Failed to count remaining entries");

        assert_eq!(remaining_count, 1, "Should have 1 entry remaining");
    }

    #[tokio::test]
    async fn test_email_cascade_delete() {
        let pool = init_test_db().await.expect("Failed to init test database");

        // Insert account
        let account_id = sqlx::query(
            "INSERT INTO accounts (email, imap_server, imap_port, smtp_server, smtp_port)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("test@example.com")
        .bind("imap.example.com")
        .bind(993)
        .bind("smtp.example.com")
        .bind(587)
        .execute(&pool)
        .await
        .expect("Failed to insert account")
        .last_insert_rowid();

        let now = chrono::Utc::now().timestamp();

        // Insert email
        sqlx::query(
            "INSERT INTO emails (account_id, folder_name, uid, subject, from_addr, to_addr, date, timestamp, synced_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(account_id)
        .bind("INBOX")
        .bind(1)
        .bind("Test Email")
        .bind("sender@example.com")
        .bind("receiver@example.com")
        .bind("Mon, 15 Jan 2024 14:30:00 +0000")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("Failed to insert email");

        // Verify email exists
        let email_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM emails WHERE account_id = ?")
            .bind(account_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count emails");

        assert_eq!(email_count, 1, "Should have 1 email");

        // Delete account (should cascade delete emails)
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(account_id)
            .execute(&pool)
            .await
            .expect("Failed to delete account");

        // Verify email is cascade deleted
        let email_count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM emails WHERE account_id = ?")
            .bind(account_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count emails after delete");

        assert_eq!(email_count_after, 0, "Emails should be cascade deleted");
    }
}
