// Notification settings and operations
use crate::db;
use tauri::command;

/// Get notification enabled setting
#[command]
pub async fn get_notification_enabled() -> Result<bool, String> {
    let pool = db::pool();
    let result = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM settings WHERE key = 'notification_enabled'",
    )
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to get notification setting: {}", e))?;

    Ok(result.0 == "true")
}

/// Set notification enabled setting
#[command]
pub async fn set_notification_enabled(enabled: bool) -> Result<(), String> {
    let pool = db::pool();
    let value = if enabled { "true" } else { "false" };

    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('notification_enabled', ?)")
        .bind(value)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to set notification setting: {}", e))?;

    Ok(())
}

/// Get sound enabled setting
#[command]
pub async fn get_sound_enabled() -> Result<bool, String> {
    let pool = db::pool();
    let result =
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'sound_enabled'")
            .fetch_one(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to get sound setting: {}", e))?;

    Ok(result.0 == "true")
}

/// Set sound enabled setting
#[command]
pub async fn set_sound_enabled(enabled: bool) -> Result<(), String> {
    let pool = db::pool();
    let value = if enabled { "true" } else { "false" };

    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('sound_enabled', ?)")
        .bind(value)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to set sound setting: {}", e))?;

    Ok(())
}
