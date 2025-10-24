// Sync interval settings management
// This module handles reading and writing sync interval preferences

use crate::db;
use tauri::command;

/// Get sync interval setting
#[command]
pub async fn get_sync_interval() -> Result<i64, String> {
    let pool = db::pool();

    let result =
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'sync_interval'")
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to get sync interval: {}", e))?;

    let interval_str = result.map(|(v,)| v).unwrap_or_else(|| "300".to_string());
    interval_str
        .parse::<i64>()
        .map_err(|e| format!("Failed to parse sync interval: {}", e))
}

/// Set sync interval setting
#[command]
pub async fn set_sync_interval(interval: i64) -> Result<(), String> {
    let pool = db::pool();

    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('sync_interval', ?)")
        .bind(interval.to_string())
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to set sync interval: {}", e))?;

    println!("✅ Set sync interval to {} seconds", interval);
    Ok(())
}
