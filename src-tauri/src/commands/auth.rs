use keyring::Entry;
use tauri::command;

const AUTH_SERVICE_NAME: &str = "com.colimail.app.auth";

/// Store a value in secure storage (OS keyring)
#[command]
pub async fn get_secure_storage(key: String) -> Result<String, String> {
    let entry = Entry::new(AUTH_SERVICE_NAME, &key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .get_password()
        .map_err(|e| format!("Failed to get value from keyring: {}", e))
}

/// Get a value from secure storage (OS keyring)
#[command]
pub async fn set_secure_storage(key: String, value: String) -> Result<(), String> {
    let entry = Entry::new(AUTH_SERVICE_NAME, &key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(&value)
        .map_err(|e| format!("Failed to store value in keyring: {}", e))
}

/// Delete a value from secure storage (OS keyring)
#[command]
pub async fn delete_secure_storage(key: String) -> Result<(), String> {
    let entry = Entry::new(AUTH_SERVICE_NAME, &key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete from keyring: {}", e))
}

/// Sync app user to local database
#[command]
pub async fn sync_app_user(
    user_id: String,
    email: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
    subscription_tier: String,
    subscription_expires_at: Option<i64>,
) -> Result<(), String> {
    let pool = crate::db::pool();

    sqlx::query(
        "INSERT INTO app_user (id, email, display_name, avatar_url, subscription_tier, subscription_expires_at, last_synced_at)
         VALUES (?, ?, ?, ?, ?, ?, strftime('%s', 'now'))
         ON CONFLICT(id) DO UPDATE SET
            email = excluded.email,
            display_name = excluded.display_name,
            avatar_url = excluded.avatar_url,
            subscription_tier = excluded.subscription_tier,
            subscription_expires_at = excluded.subscription_expires_at,
            last_synced_at = excluded.last_synced_at",
    )
    .bind(&user_id)
    .bind(&email)
    .bind(&display_name)
    .bind(&avatar_url)
    .bind(&subscription_tier)
    .bind(subscription_expires_at)
    .execute(&*pool)
    .await
    .map_err(|e| format!("Failed to sync user to database: {}", e))?;

    Ok(())
}

/// Get app user from local database
#[command]
pub async fn get_app_user(user_id: String) -> Result<Option<AppUserData>, String> {
    let pool = crate::db::pool();

    let result = sqlx::query_as::<_, AppUserData>(
        "SELECT id, email, display_name, avatar_url, subscription_tier, subscription_expires_at, created_at
         FROM app_user
         WHERE id = ?",
    )
    .bind(&user_id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| format!("Failed to get user from database: {}", e))?;

    Ok(result)
}

/// Delete app user from local database
#[command]
pub async fn delete_app_user(user_id: String) -> Result<(), String> {
    let pool = crate::db::pool();

    sqlx::query("DELETE FROM app_user WHERE id = ?")
        .bind(&user_id)
        .execute(&*pool)
        .await
        .map_err(|e| format!("Failed to delete user from database: {}", e))?;

    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AppUserData {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub subscription_tier: String,
    pub subscription_expires_at: Option<i64>,
    pub created_at: i64,
}
