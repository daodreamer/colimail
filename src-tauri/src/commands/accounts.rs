use crate::db::pool;
use crate::models::{AccountConfig, AuthType};
use crate::security;
use tauri::command;

#[command]
pub async fn delete_account(email: String) -> Result<(), String> {
    let pool = pool();

    // Delete from database
    sqlx::query("DELETE FROM accounts WHERE email = ?")
        .bind(&email)
        .execute(&*pool)
        .await
        .map_err(|e| format!("Failed to delete account: {}", e))?;

    // Delete credentials from keyring
    if let Err(e) = security::delete_credentials(&email) {
        eprintln!(
            "⚠️  Warning: Failed to delete credentials from keyring: {}",
            e
        );
    }

    println!("✅ Account deleted: {}", email);
    Ok(())
}

#[command]
pub async fn save_account_config(config: AccountConfig) -> Result<(), String> {
    let pool = pool();

    let auth_type = match config.auth_type {
        Some(AuthType::OAuth2) => "oauth2",
        _ => "basic",
    };

    // Save non-sensitive data to database
    // If config has an ID, use UPDATE to preserve the ID
    // Otherwise use INSERT to create new account
    if let Some(id) = config.id {
        // Update existing account
        sqlx::query(
            "UPDATE accounts
             SET imap_server = ?, imap_port = ?, smtp_server = ?, smtp_port = ?,
                 auth_type = ?, display_name = ?
             WHERE id = ?",
        )
        .bind(&config.imap_server)
        .bind(config.imap_port as i64)
        .bind(&config.smtp_server)
        .bind(config.smtp_port as i64)
        .bind(auth_type)
        .bind(&config.display_name)
        .bind(id as i64)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        // Insert new account
        sqlx::query(
            "INSERT INTO accounts
             (email, imap_server, imap_port, smtp_server, smtp_port, auth_type, display_name)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&config.email)
        .bind(&config.imap_server)
        .bind(config.imap_port as i64)
        .bind(&config.smtp_server)
        .bind(config.smtp_port as i64)
        .bind(auth_type)
        .bind(&config.display_name)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Save sensitive credentials to OS keyring
    let credentials = security::AccountCredentials {
        email: config.email.clone(),
        password: config.password,
        access_token: config.access_token,
        refresh_token: config.refresh_token,
        token_expires_at: config.token_expires_at,
    };

    security::store_credentials(&credentials)?;

    println!("✅ Account saved securely: {}", config.email);
    Ok(())
}

#[command]
pub async fn load_account_configs() -> Result<Vec<AccountConfig>, String> {
    let pool = pool();

    // Load non-sensitive data from database
    let accounts = sqlx::query_as::<_, (i64, String, String, i64, String, i64, String, Option<String>)>(
        "SELECT id, email, imap_server, imap_port, smtp_server, smtp_port, auth_type, display_name FROM accounts",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .map(
        |(id, email, imap_server, imap_port, smtp_server, smtp_port, auth_type, display_name)| {
            let auth_type_enum = match auth_type.as_str() {
                "oauth2" => Some(AuthType::OAuth2),
                _ => Some(AuthType::Basic),
            };

            // Load sensitive credentials from keyring
            match security::get_credentials(&email) {
                Ok(creds) => AccountConfig {
                    id: Some(id as i32),
                    email,
                    password: creds.password,
                    imap_server,
                    imap_port: imap_port as u16,
                    smtp_server,
                    smtp_port: smtp_port as u16,
                    auth_type: auth_type_enum,
                    access_token: creds.access_token,
                    refresh_token: creds.refresh_token,
                    token_expires_at: creds.token_expires_at,
                    display_name,
                },
                Err(e) => {
                    eprintln!("⚠️  Failed to load credentials for {}: {}", email, e);
                    // Return account config without credentials
                    AccountConfig {
                        id: Some(id as i32),
                        email,
                        password: None,
                        imap_server,
                        imap_port: imap_port as u16,
                        smtp_server,
                        smtp_port: smtp_port as u16,
                        auth_type: auth_type_enum,
                        access_token: None,
                        refresh_token: None,
                        token_expires_at: None,
                        display_name,
                    }
                }
            }
        },
    )
    .collect();

    Ok(accounts)
}
