use crate::db::pool;
use crate::models::{AccountConfig, AuthType};
use tauri::command;

#[command]
pub async fn delete_account(email: String) -> Result<(), String> {
    let pool = pool();

    sqlx::query("DELETE FROM accounts WHERE email = ?")
        .bind(&email)
        .execute(&*pool)
        .await
        .map_err(|e| format!("Failed to delete account: {}", e))?;

    println!("✅ Account deleted from database: {}", email);
    Ok(())
}

#[command]
pub async fn save_account_config(config: AccountConfig) -> Result<(), String> {
    let pool = pool();

    let auth_type = match config.auth_type {
        Some(AuthType::OAuth2) => "oauth2",
        _ => "basic",
    };

    sqlx::query(
        "INSERT OR REPLACE INTO accounts
         (email, password, imap_server, imap_port, smtp_server, smtp_port, auth_type, access_token, refresh_token, token_expires_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&config.email)
    .bind(&config.password)
    .bind(&config.imap_server)
    .bind(config.imap_port as i64)
    .bind(&config.smtp_server)
    .bind(config.smtp_port as i64)
    .bind(auth_type)
    .bind(&config.access_token)
    .bind(&config.refresh_token)
    .bind(config.token_expires_at)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    println!("✅ Account saved to database: {}", config.email);
    Ok(())
}

#[command]
pub async fn load_account_configs() -> Result<Vec<AccountConfig>, String> {
    let pool = pool();

    let accounts = sqlx::query_as::<_, (i64, String, Option<String>, String, i64, String, i64, String, Option<String>, Option<String>, Option<i64>)>(
        "SELECT id, email, password, imap_server, imap_port, smtp_server, smtp_port, auth_type, access_token, refresh_token, token_expires_at FROM accounts",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .map(
        |(id, email, password, imap_server, imap_port, smtp_server, smtp_port, auth_type, access_token, refresh_token, token_expires_at)| {
            let auth_type_enum = match auth_type.as_str() {
                "oauth2" => Some(AuthType::OAuth2),
                _ => Some(AuthType::Basic),
            };

            AccountConfig {
                id: Some(id as i32),
                email,
                password,
                imap_server,
                imap_port: imap_port as u16,
                smtp_server,
                smtp_port: smtp_port as u16,
                auth_type: auth_type_enum,
                access_token,
                refresh_token,
                token_expires_at,
            }
        },
    )
    .collect();

    Ok(accounts)
}
