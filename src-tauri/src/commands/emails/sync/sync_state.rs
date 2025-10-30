// Sync state management
// This module handles storing and retrieving sync state for folders

use crate::db;
use chrono::Utc;
use tauri::command;

/// Struct to hold sync state from database
#[derive(Clone)]
pub struct SyncState {
    pub uidvalidity: Option<i64>,
    pub highest_uid: Option<i64>,
}

/// Get sync state for a folder
pub async fn get_sync_state(
    account_id: i32,
    folder_name: &str,
) -> Result<Option<SyncState>, String> {
    let pool = db::pool();

    let result = sqlx::query_as::<_, (Option<i64>, Option<i64>)>(
        "SELECT uidvalidity, highest_uid FROM sync_status WHERE account_id = ? AND folder_name = ?",
    )
    .bind(account_id)
    .bind(folder_name)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to get sync state: {}", e))?;

    Ok(result.map(|(uidvalidity, highest_uid)| SyncState {
        uidvalidity,
        highest_uid,
    }))
}

/// Update sync state for a folder
pub async fn update_sync_state(
    account_id: i32,
    folder_name: &str,
    uidvalidity: i64,
    highest_uid: i64,
) -> Result<(), String> {
    let pool = db::pool();
    let current_time = Utc::now().timestamp();

    sqlx::query(
        "INSERT OR REPLACE INTO sync_status (account_id, folder_name, last_sync_time, uidvalidity, highest_uid)
        VALUES (?, ?, ?, ?, ?)",
    )
    .bind(account_id)
    .bind(folder_name)
    .bind(current_time)
    .bind(uidvalidity)
    .bind(highest_uid)
    .execute(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to update sync state: {}", e))?;

    println!(
        "✅ Updated sync state: UIDVALIDITY={}, highest_uid={}",
        uidvalidity, highest_uid
    );

    Ok(())
}

/// Get last sync time for a folder
#[command]
pub async fn get_last_sync_time(account_id: i32, folder: Option<String>) -> Result<i64, String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    let pool = db::pool();

    let result = sqlx::query_as::<_, (i64,)>(
        "SELECT last_sync_time FROM sync_status WHERE account_id = ? AND folder_name = ?",
    )
    .bind(account_id)
    .bind(&folder_name)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to get last sync time: {}", e))?;

    Ok(result.map(|(time,)| time).unwrap_or(0))
}

/// Check if sync is needed based on interval
#[command]
pub async fn should_sync(
    account_id: i32,
    folder: Option<String>,
    sync_interval: i64,
) -> Result<bool, String> {
    // sync_interval in seconds, 0 = manual, -1 = never
    if sync_interval == -1 {
        return Ok(false); // Never sync
    }
    if sync_interval == 0 {
        return Ok(false); // Manual only
    }

    let last_sync = get_last_sync_time(account_id, folder).await?;
    let current_time = Utc::now().timestamp();
    let elapsed = current_time - last_sync;

    Ok(elapsed >= sync_interval)
}
