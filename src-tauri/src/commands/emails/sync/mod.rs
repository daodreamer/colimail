// Email synchronization module
// This module coordinates incremental sync using UIDVALIDITY and UIDs

// Sub-modules
mod parse;
mod sync_core;
mod sync_fetch;
mod sync_flags;
mod sync_state;

// Re-export public command functions
pub use sync_flags::{sync_email_flags, sync_specific_email_flags};
pub use sync_state::{get_last_sync_time, should_sync};

use crate::commands::emails::cache::load_emails_from_cache;
use crate::commands::emails::fetch_bodystructure;
use crate::models::{AccountConfig, EmailHeader};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::command;

/// Sync emails from server and update cache (incremental sync)
#[command]
pub async fn sync_emails(
    config: AccountConfig,
    folder: Option<String>,
) -> Result<Vec<EmailHeader>, String> {
    let account_id = config.id.ok_or("Account ID is required")?;
    let folder_name = folder.clone().unwrap_or_else(|| "INBOX".to_string());

    println!(
        "🔄 Starting incremental sync for account {} folder {}",
        account_id, folder_name
    );

    // Perform incremental sync
    let new_emails = sync_core::incremental_sync(config.clone(), account_id, &folder_name).await?;

    println!(
        "✅ Incremental sync completed: fetched {} new emails",
        new_emails.len()
    );

    // Sync flags for existing emails (to catch flag changes from other clients)
    // This ensures star/read status stays in sync with server
    println!("🔄 Syncing flags for existing emails in background...");
    let config_for_flag_sync = config.clone();
    let account_id_for_flag_sync = account_id;
    let folder_name_for_flag_sync = folder_name.clone();

    // Run flag sync in background to avoid blocking
    tokio::spawn(async move {
        if let Err(e) = sync_flags::sync_email_flags(
            account_id_for_flag_sync,
            &folder_name_for_flag_sync,
            config_for_flag_sync,
        )
        .await
        {
            eprintln!("⚠️ Background flag sync failed: {}", e);
        }
    });

    // Load all cached emails (for display)
    let emails = load_emails_from_cache(account_id, Some(folder_name.clone())).await?;

    println!("✅ Sync completed: {} emails in cache total", emails.len());

    // Start background task to fetch BODYSTRUCTURE (newest first)
    // This improves perceived performance by showing emails immediately
    let account_id_i64 = account_id as i64;
    let folder_name_clone = folder_name.clone();
    let cancel_token = Arc::new(AtomicBool::new(false));

    tokio::spawn(async move {
        if let Err(e) = fetch_bodystructure::fetch_bodystructure_background(
            account_id_i64,
            folder_name_clone,
            cancel_token,
        )
        .await
        {
            eprintln!("⚠️ Background BODYSTRUCTURE fetch failed: {}", e);
        }
    });

    Ok(emails)
}
