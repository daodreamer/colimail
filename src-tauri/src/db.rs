use directories::ProjectDirs;
use lazy_static::lazy_static;
use rusqlite::{Connection, Result};
use std::sync::{Mutex, MutexGuard};

lazy_static! {
    /// Shared SQLite connection guarded by a mutex for cross-thread access.
    pub static ref DB_CONNECTION: Mutex<Connection> = {
        let proj_dirs = ProjectDirs::from("com", "MailDesk", "MailDesk")
            .expect("Failed to determine project directories");
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
        let db_path = data_dir.join("maildesk.db");

        let conn = Connection::open(db_path).expect("Failed to open database");
        Mutex::new(conn)
    };
}

/// Initialize the database schema if it does not exist yet.
pub fn init() -> Result<()> {
    let conn = connection();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            imap_server TEXT NOT NULL,
            imap_port INTEGER NOT NULL,
            smtp_server TEXT NOT NULL,
            smtp_port INTEGER NOT NULL
        )",
        (),
    )?;
    Ok(())
}

/// Helper to lock and access the global connection.
pub fn connection() -> MutexGuard<'static, Connection> {
    DB_CONNECTION
        .lock()
        .expect("Failed to lock database connection")
}
