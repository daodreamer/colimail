// Email deletion operations
// This module handles moving emails to trash and permanent deletion

use crate::commands::emails::fetch::OAuth2;
use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType};
use native_tls::TlsConnector;
use tauri::command;

/// Helper function to find the trash/deleted folder for an account
/// Different providers use different names for the trash folder
fn find_trash_folder(
    imap_session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
) -> Result<String, String> {
    // Common trash folder names used by different providers
    let trash_candidates = vec![
        "[Gmail]/Trash",           // Gmail
        "[Gmail]/Bin",             // Gmail (some locales)
        "Trash",                   // Standard IMAP
        "Deleted",                 // Common alternative
        "Deleted Items",           // Outlook/Exchange
        "Deleted Messages",        // Some providers
        "[Outlook]/Deleted Items", // Outlook with prefix
        "INBOX.Trash",             // Some IMAP servers
        "INBOX.Deleted",           // Some IMAP servers
        "垃圾箱",                  // Chinese
        "已删除邮件",              // Chinese (Outlook style)
    ];

    println!("Searching for trash folder...");

    // List all folders
    let mailbox_list = imap_session
        .list(Some(""), Some("*"))
        .map_err(|e| format!("Failed to list folders: {}", e))?;

    // Try to find a trash folder by matching against known names
    for mailbox in mailbox_list.iter() {
        let folder_name = mailbox.name();
        let lower_name = folder_name.to_lowercase();

        // Check if this folder matches any of our trash candidates
        for candidate in &trash_candidates {
            if folder_name == *candidate
                || lower_name.contains("trash")
                || lower_name.contains("deleted")
                || lower_name.contains("垃圾")
            {
                // Verify the folder is selectable
                if !mailbox
                    .attributes()
                    .iter()
                    .any(|attr| matches!(attr, imap::types::NameAttribute::NoSelect))
                {
                    println!("Found trash folder: {}", folder_name);
                    return Ok(folder_name.to_string());
                }
            }
        }
    }

    // If no trash folder found, return an error
    Err("Could not find trash/deleted folder. The email provider may not have a standard trash folder.".to_string())
}

/// Move email to trash folder (soft delete)
#[command]
pub async fn move_email_to_trash(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<(), String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    let account_id = config.id.ok_or("Account ID is required")?;
    println!("Moving email UID {} from {} to trash", uid, folder_name);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Clone folder_name for use in both the blocking task and later cache removal
    let folder_name_for_task = folder_name.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let folder_name = folder_name_for_task;
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

                println!("🔐 Attempting OAuth2 authentication for: {}", email);

                let oauth2 = OAuth2 {
                    user: email.to_string(),
                    access_token: access_token.clone(),
                };

                client.authenticate("XOAUTH2", &oauth2).map_err(|e| {
                    eprintln!("❌ OAuth2 authentication failed: {}", e.0);
                    format!("OAuth2 authentication failed: {}", e.0)
                })?
            }
            _ => {
                let password = config
                    .password
                    .as_ref()
                    .ok_or("Password is required for basic authentication")?;

                client.login(email, password).map_err(|e| e.0.to_string())?
            }
        };

        println!("IMAP authentication successful");

        // Find the trash folder
        let trash_folder = find_trash_folder(&mut imap_session)?;
        println!("Using trash folder: {}", trash_folder);

        // Select the source folder
        imap_session.select(&folder_name).map_err(|e| {
            eprintln!("❌ Failed to SELECT folder '{}': {}", folder_name, e);
            format!("Cannot access folder '{}': {}", folder_name, e)
        })?;

        // Copy the email to trash folder using UID COPY
        imap_session
            .uid_copy(format!("{}", uid), &trash_folder)
            .map_err(|e| {
                eprintln!("❌ Failed to copy UID {} to trash: {}", uid, e);
                format!("Failed to copy email to trash: {}", e)
            })?;

        println!("Copied UID {} to trash folder", uid);

        // Mark the original email for deletion using UID STORE
        imap_session
            .uid_store(format!("{}", uid), "+FLAGS (\\Deleted)")
            .map_err(|e| {
                eprintln!("❌ Failed to mark UID {} as deleted: {}", uid, e);
                format!("Failed to mark email as deleted: {}", e)
            })?;

        println!("Marked UID {} with \\Deleted flag", uid);

        // Permanently delete (expunge) the original
        imap_session.expunge().map_err(|e| {
            eprintln!("❌ Failed to expunge deleted messages: {}", e);
            format!("Failed to remove email from original folder: {}", e)
        })?;

        println!("✅ Successfully moved email to trash");

        let _ = imap_session.logout();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    // Remove email from local cache immediately for responsive UI
    let pool = crate::db::pool();
    sqlx::query("DELETE FROM emails WHERE account_id = ? AND folder_name = ? AND uid = ?")
        .bind(account_id)
        .bind(&folder_name)
        .bind(uid as i64)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to remove email from cache: {}", e))?;

    println!(
        "✅ Successfully moved email UID {} to trash and removed from cache",
        uid
    );

    Ok(())
}

/// Permanently delete email (hard delete)
#[command]
pub async fn delete_email(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<(), String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    let account_id = config.id.ok_or("Account ID is required")?;
    println!(
        "Permanently deleting email UID {} from {}",
        uid, folder_name
    );

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Clone folder_name for use in both the blocking task and later cache removal
    let folder_name_for_task = folder_name.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let folder_name = folder_name_for_task;
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

                println!("🔐 Attempting OAuth2 authentication for: {}", email);

                let oauth2 = OAuth2 {
                    user: email.to_string(),
                    access_token: access_token.clone(),
                };

                client.authenticate("XOAUTH2", &oauth2).map_err(|e| {
                    eprintln!("❌ OAuth2 authentication failed: {}", e.0);
                    format!("OAuth2 authentication failed: {}", e.0)
                })?
            }
            _ => {
                let password = config
                    .password
                    .as_ref()
                    .ok_or("Password is required for basic authentication")?;

                client.login(email, password).map_err(|e| e.0.to_string())?
            }
        };

        println!("IMAP authentication successful");

        // Select the folder
        imap_session.select(&folder_name).map_err(|e| {
            eprintln!("❌ Failed to SELECT folder '{}': {}", folder_name, e);
            format!("Cannot access folder '{}': {}", folder_name, e)
        })?;

        // Mark the email for deletion using UID
        imap_session
            .uid_store(format!("{}", uid), "+FLAGS (\\Deleted)")
            .map_err(|e| {
                eprintln!("❌ Failed to mark UID {} as deleted: {}", uid, e);
                format!("Failed to mark email as deleted: {}", e)
            })?;

        println!("Marked UID {} with \\Deleted flag", uid);

        // Permanently delete messages marked with \Deleted flag
        imap_session.expunge().map_err(|e| {
            eprintln!("❌ Failed to expunge deleted messages: {}", e);
            format!("Failed to permanently delete email: {}", e)
        })?;

        println!("✅ Successfully expunged deleted messages");

        let _ = imap_session.logout();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    // Remove email from local cache immediately for responsive UI
    let pool = crate::db::pool();
    sqlx::query("DELETE FROM emails WHERE account_id = ? AND folder_name = ? AND uid = ?")
        .bind(account_id)
        .bind(&folder_name)
        .bind(uid as i64)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to remove email from cache: {}", e))?;

    println!(
        "✅ Successfully deleted email UID {} from server and removed from cache",
        uid
    );

    Ok(())
}
