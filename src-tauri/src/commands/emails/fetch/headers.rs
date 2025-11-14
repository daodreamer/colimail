//! Raw email headers fetching operations
//!
//! This module handles fetching raw email headers from IMAP servers,
//! used primarily for CMVH (ColiMail Verification Header) signature verification.

use crate::commands::emails::cache::{load_raw_headers_from_cache, save_raw_headers_to_cache};
use crate::commands::emails::imap_helpers;
use crate::commands::utils::ensure_valid_token;
use crate::models::AccountConfig;
use tauri::command;

/// Fetch raw email headers (for CMVH verification) with caching
#[command]
pub async fn fetch_email_raw_headers(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<String, String> {
    let account_id = config.id.ok_or("Account ID is required")?;
    let folder_name = folder.clone().unwrap_or_else(|| "INBOX".to_string());

    println!(
        "🔍 fetch_email_raw_headers called: account_id={}, uid={}, folder={}",
        account_id, uid, folder_name
    );

    // Try to load from cache first
    if let Some(cached_headers) = load_raw_headers_from_cache(account_id, &folder_name, uid).await?
    {
        println!("✅ Loaded raw headers from cache for UID {}", uid);
        return Ok(cached_headers);
    }

    println!(
        "📥 Cache miss - fetching raw headers from server for UID {}",
        uid
    );

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Clone folder_name for the blocking task
    let folder_for_task = folder_name.clone();

    let raw_headers = tokio::task::spawn_blocking(move || -> Result<String, String> {
        // Use helper function for connection with imap 3.0.0 API
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

        imap_session.select(&folder_for_task).map_err(|e| {
            eprintln!(
                "❌ Failed to SELECT folder '{}' for UID {}: {}",
                folder_for_task, uid, e
            );
            format!("Cannot access folder '{}': {}", folder_for_task, e)
        })?;

        let messages = imap_session
            .uid_fetch(uid.to_string(), "BODY[HEADER]")
            .map_err(|e| {
                eprintln!("❌ UID FETCH failed for UID {}: {}", uid, e);
                e.to_string()
            })?;

        // In imap 3.0.0, Fetches implements Iterator, use iter().next() instead of first()
        let message = messages.iter().next().ok_or_else(|| {
            eprintln!("❌ No message found for UID {}", uid);
            "No message found for UID".to_string()
        })?;

        let raw_headers = message.body().unwrap_or_default();
        let headers_str = String::from_utf8_lossy(raw_headers).to_string();

        let _ = imap_session.logout();
        Ok(headers_str)
    })
    .await
    .map_err(|e| e.to_string())??;

    // Save to cache
    save_raw_headers_to_cache(account_id, &folder_name, uid, &raw_headers).await?;

    println!(
        "✅ Successfully fetched and cached raw headers for UID {}",
        uid
    );
    Ok(raw_headers)
}
