use keyring::Entry;
use tauri::command;

const WALLET_SERVICE_NAME: &str = "com.colimail.app.wallet";

/// Wallet session data stored in OS keyring
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WalletSession {
    pub address: String,
    pub chain_id: u64,
    pub connection_method: String, // "walletconnect" only - browser wallets not supported for security
    pub last_active_timestamp: i64, // Unix timestamp
    pub wc_session_topic: Option<String>, // WalletConnect session topic (for restoration)
}

/// Get wallet session timeout setting (in seconds)
#[command]
pub async fn get_wallet_session_timeout() -> Result<i64, String> {
    let pool = crate::db::pool();

    let result = sqlx::query_scalar::<_, String>(
        "SELECT value FROM settings WHERE key = 'wallet_session_timeout'",
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|e| format!("Failed to get wallet session timeout: {}", e))?;

    // Default: 24 hours (86400 seconds)
    match result {
        Some(value_str) => value_str
            .parse::<i64>()
            .map_err(|e| format!("Failed to parse timeout value: {}", e)),
        None => Ok(86400),
    }
}

/// Set wallet session timeout setting (in seconds)
#[command]
pub async fn set_wallet_session_timeout(timeout_seconds: i64) -> Result<(), String> {
    let pool = crate::db::pool();

    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('wallet_session_timeout', ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(timeout_seconds.to_string())
    .execute(&*pool)
    .await
    .map_err(|e| format!("Failed to set wallet session timeout: {}", e))?;

    Ok(())
}

/// Save wallet session to OS keyring
#[command]
pub async fn save_wallet_session(session: WalletSession) -> Result<(), String> {
    let entry = Entry::new(WALLET_SERVICE_NAME, "active_session")
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    let session_json = serde_json::to_string(&session)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;

    entry
        .set_password(&session_json)
        .map_err(|e| format!("Failed to store session in keyring: {}", e))
}

/// Get wallet session from OS keyring (returns None if expired or not found)
#[command]
pub async fn get_wallet_session() -> Result<Option<WalletSession>, String> {
    let entry = Entry::new(WALLET_SERVICE_NAME, "active_session")
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    let session_json = match entry.get_password() {
        Ok(json) => json,
        Err(_) => return Ok(None), // No session found
    };

    let session: WalletSession = serde_json::from_str(&session_json)
        .map_err(|e| format!("Failed to deserialize session: {}", e))?;

    // Check if session is expired
    let timeout = get_wallet_session_timeout().await?;
    let now = chrono::Utc::now().timestamp();

    if timeout > 0 && now - session.last_active_timestamp > timeout {
        // Session expired, delete it
        delete_wallet_session().await?;
        return Ok(None);
    }

    Ok(Some(session))
}

/// Delete wallet session from OS keyring
#[command]
pub async fn delete_wallet_session() -> Result<(), String> {
    let entry = Entry::new(WALLET_SERVICE_NAME, "active_session")
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete session from keyring: {}", e))
}

/// Update last active timestamp for current session
#[command]
pub async fn update_wallet_session_activity() -> Result<(), String> {
    if let Some(mut session) = get_wallet_session().await? {
        session.last_active_timestamp = chrono::Utc::now().timestamp();
        save_wallet_session(session).await?;
    }
    Ok(())
}
