// Email flag operations (mark as read/unread)
// This module handles setting IMAP flags and syncing with the server

use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::{AccountConfig, AuthType};
use native_tls::TlsConnector;
use tauri::command;

/// OAuth2 authenticator for IMAP
pub struct OAuth2 {
    pub user: String,
    pub access_token: String,
}

impl imap::Authenticator for OAuth2 {
    type Response = String;
    fn process(&self, _: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}

/// Mark email as read (set \Seen flag) on IMAP server and update local cache
#[command]
pub async fn mark_email_as_read(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<(), String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    let account_id = config.id.ok_or("Account ID is required")?;

    println!(
        "Marking email UID {} as read in folder {}",
        uid, folder_name
    );

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Clone folder_name for the database query
    let folder_name_for_db = folder_name.clone();

    // Mark as read on IMAP server
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let domain = config.imap_server.as_str();
        let port = config.imap_port;
        let email = config.email.as_str();

        let tls = TlsConnector::builder().build().map_err(|e| e.to_string())?;
        let client = imap::connect((domain, port), domain, &tls).map_err(|e| e.to_string())?;

        let mut imap_session = match config.auth_type {
            Some(AuthType::OAuth2) => {
                let access_token = config
                    .access_token
                    .as_ref()
                    .ok_or("Access token is required for OAuth2 authentication")?;

                let oauth2 = OAuth2 {
                    user: email.to_string(),
                    access_token: access_token.clone(),
                };

                client
                    .authenticate("XOAUTH2", &oauth2)
                    .map_err(|e| format!("OAuth2 authentication failed: {}", e.0))?
            }
            _ => {
                let password = config
                    .password
                    .as_ref()
                    .ok_or("Password is required for basic authentication")?;

                client.login(email, password).map_err(|e| e.0.to_string())?
            }
        };

        imap_session
            .select(&folder_name)
            .map_err(|e| format!("Cannot access folder '{}': {}", folder_name, e))?;

        // Add \Seen flag to the email
        imap_session
            .uid_store(uid.to_string(), "+FLAGS (\\Seen)")
            .map_err(|e| format!("Failed to set \\Seen flag: {}", e))?;

        let _ = imap_session.logout();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    // Update local cache
    let pool = db::pool();
    sqlx::query("UPDATE emails SET seen = 1 WHERE account_id = ? AND folder_name = ? AND uid = ?")
        .bind(account_id)
        .bind(&folder_name_for_db)
        .bind(uid as i64)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to update local cache: {}", e))?;

    println!("✅ Marked email UID {} as read", uid);
    Ok(())
}

/// Mark email as unread (remove \Seen flag) on IMAP server and update local cache
#[command]
pub async fn mark_email_as_unread(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<(), String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    let account_id = config.id.ok_or("Account ID is required")?;

    println!(
        "Marking email UID {} as unread in folder {}",
        uid, folder_name
    );

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Clone folder_name for the database query
    let folder_name_for_db = folder_name.clone();

    // Mark as unread on IMAP server
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let domain = config.imap_server.as_str();
        let port = config.imap_port;
        let email = config.email.as_str();

        let tls = TlsConnector::builder().build().map_err(|e| e.to_string())?;
        let client = imap::connect((domain, port), domain, &tls).map_err(|e| e.to_string())?;

        let mut imap_session = match config.auth_type {
            Some(AuthType::OAuth2) => {
                let access_token = config
                    .access_token
                    .as_ref()
                    .ok_or("Access token is required for OAuth2 authentication")?;

                let oauth2 = OAuth2 {
                    user: email.to_string(),
                    access_token: access_token.clone(),
                };

                client
                    .authenticate("XOAUTH2", &oauth2)
                    .map_err(|e| format!("OAuth2 authentication failed: {}", e.0))?
            }
            _ => {
                let password = config
                    .password
                    .as_ref()
                    .ok_or("Password is required for basic authentication")?;

                client.login(email, password).map_err(|e| e.0.to_string())?
            }
        };

        imap_session
            .select(&folder_name)
            .map_err(|e| format!("Cannot access folder '{}': {}", folder_name, e))?;

        // Remove \Seen flag from the email
        imap_session
            .uid_store(uid.to_string(), "-FLAGS (\\Seen)")
            .map_err(|e| format!("Failed to remove \\Seen flag: {}", e))?;

        let _ = imap_session.logout();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    // Update local cache
    let pool = db::pool();
    sqlx::query("UPDATE emails SET seen = 0 WHERE account_id = ? AND folder_name = ? AND uid = ?")
        .bind(account_id)
        .bind(&folder_name_for_db)
        .bind(uid as i64)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to update local cache: {}", e))?;

    println!("✅ Marked email UID {} as unread", uid);
    Ok(())
}
