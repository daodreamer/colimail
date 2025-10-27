// Email flag operations (mark as read/unread)
// This module handles setting IMAP flags and syncing with the server

use crate::commands::emails::imap_helpers;
use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::AccountConfig;
use tauri::command;

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
        // Use helper function for connection with imap 3.0.0 API
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

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
        // Use helper function for connection with imap 3.0.0 API
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

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
