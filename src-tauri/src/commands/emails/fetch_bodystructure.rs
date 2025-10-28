use crate::commands::emails::imap_helpers;
use crate::db;
use crate::models::{AccountConfig, AuthType};
use crate::security;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Check if an email has attachments by fetching its BODYSTRUCTURE
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub uid: u32,
    pub has_attachments: bool,
    pub attachment_count: usize,
}

/// Background task to fetch BODYSTRUCTURE for emails (newest first)
///
/// This improves initial load time by deferring attachment detection to a background task.
/// Inspired by Thunderbird's nsAutoSyncManager approach:
/// 1. Initial sync fetches basic headers without BODYSTRUCTURE (fast)
/// 2. Background task fetches BODYSTRUCTURE to detect attachments (accurate but slower)
///
/// # Retry Logic
/// - Batch processing: 5 emails per batch with 100ms delay
/// - Failed batches are tracked and retried individually
/// - Individual retry handles connection errors (reconnect) and format errors (mark as no attachments)
/// - Prevents emails from getting stuck in "pending" state forever
///
/// # Connection Management
/// - Preventive reconnection: every 100 batches to avoid server timeouts
/// - Reactive reconnection: on Bye/TagMismatch/Connection errors
/// - Graceful degradation: continues with existing session if preventive reconnect fails
pub async fn fetch_bodystructure_background(
    account_id: i64,
    folder_name: String,
    cancel_token: Arc<AtomicBool>,
) -> Result<(), String> {
    println!(
        "🔄 Starting background BODYSTRUCTURE fetch for account {} folder '{}'",
        account_id, folder_name
    );

    let pool = db::pool();

    // Get account config
    let (id, email, imap_server, imap_port, smtp_server, smtp_port, auth_type) =
        sqlx::query_as::<_, (i64, String, String, i64, String, i64, String)>(
            "SELECT id, email, imap_server, imap_port, smtp_server, smtp_port, auth_type FROM accounts WHERE id = ?"
        )
        .bind(account_id)
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| format!("Account {} not found: {}", account_id, e))?;

    let auth_type_enum = match auth_type.as_str() {
        "oauth2" => Some(AuthType::OAuth2),
        _ => Some(AuthType::Basic),
    };

    // Load sensitive credentials from keyring
    let creds = security::get_credentials(&email)
        .map_err(|e| format!("Failed to load credentials for {}: {}", email, e))?;

    let config = AccountConfig {
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
    };

    // Get all UIDs that don't have attachment info yet (has_attachments IS NULL)
    let pending_uids: Vec<i64> = sqlx::query_as::<_, (i64,)>(
        "SELECT uid FROM emails 
         WHERE account_id = ? AND folder_name = ? AND has_attachments IS NULL 
         ORDER BY uid DESC", // Newest first
    )
    .bind(account_id)
    .bind(&folder_name)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to query pending UIDs: {}", e))?
    .into_iter()
    .map(|(uid,)| uid)
    .collect();

    if pending_uids.is_empty() {
        println!("✅ No emails need BODYSTRUCTURE fetch");
        return Ok(());
    }

    println!(
        "📋 Found {} emails needing BODYSTRUCTURE fetch",
        pending_uids.len()
    );

    // Spawn blocking task for IMAP operations
    let folder_name_clone = folder_name.clone();
    let config_clone = config.clone();

    let results =
        tokio::task::spawn_blocking(move || -> Result<Vec<(u32, bool, usize)>, String> {
            // Connect to IMAP
            let mut imap_session = imap_helpers::connect_and_login(&config_clone)
                .map_err(|e| format!("Failed to connect to IMAP: {}", e))?;

            // Select folder
            imap_session
                .select(&folder_name_clone)
                .map_err(|e| format!("Failed to select folder '{}': {}", folder_name_clone, e))?;

            let mut results = Vec::new();
            let mut failed_uids = Vec::new(); // Track UIDs that failed batch processing

            // Process in small batches (5 at a time) to avoid overwhelming the server
            let batch_size = 5;
            let reconnect_interval = 100; // Reconnect every 100 batches to keep connection fresh

            for (batch_idx, uid_chunk) in pending_uids.chunks(batch_size).enumerate() {
                // Check if task was cancelled
                if cancel_token.load(Ordering::Relaxed) {
                    println!("⏹️ Background BODYSTRUCTURE fetch cancelled");
                    break;
                }

                let batch_num = batch_idx + 1;

                // Reconnect every N batches to keep connection fresh and avoid timeouts
                if batch_num % reconnect_interval == 0 {
                    println!(
                        "  🔄 Reconnecting to maintain fresh connection (batch {})...",
                        batch_num
                    );
                    match reconnect(&config_clone, &folder_name_clone) {
                        Ok(new_session) => {
                            imap_session = new_session;
                            println!("  ✅ Reconnected successfully");
                        }
                        Err(e) => {
                            eprintln!("  ❌ Failed to reconnect: {}", e);
                            // Continue with existing session, might still work
                        }
                    }
                }

                // Fetch BODYSTRUCTURE for this batch
                let uid_list = uid_chunk
                    .iter()
                    .map(|uid| uid.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                match imap_session.uid_fetch(&uid_list, "(UID BODYSTRUCTURE)") {
                    Ok(messages) => {
                        for msg in messages.iter() {
                            if let Some(uid) = msg.uid {
                                let (has_attachments, attachment_count) = check_attachments(msg);
                                results.push((uid, has_attachments, attachment_count));
                            }
                        }

                        if batch_num % 10 == 0 {
                            println!(
                                "  📦 Processed {} batches ({} emails)",
                                batch_num,
                                results.len()
                            );
                        }
                    }
                    Err(e) => {
                        let error_str = format!("{:?}", e);
                        eprintln!(
                            "⚠️ Failed to fetch BODYSTRUCTURE for batch {}: {}",
                            batch_num, e
                        );

                        // Add failed UIDs to retry list
                        failed_uids.extend_from_slice(uid_chunk);

                        // Check if this is a connection error (Bye or network issue)
                        if error_str.contains("Bye")
                            || error_str.contains("TagMismatch")
                            || error_str.contains("Connection")
                        {
                            eprintln!("  🔌 Connection issue detected, attempting to reconnect...");

                            // Try to reconnect
                            match reconnect(&config_clone, &folder_name_clone) {
                                Ok(new_session) => {
                                    imap_session = new_session;
                                    println!("  ✅ Reconnected successfully");
                                }
                                Err(e) => {
                                    eprintln!("  ❌ Failed to reconnect: {}", e);
                                    // If reconnect fails, break the loop to avoid cascading failures
                                    break;
                                }
                            }

                            // Small delay after reconnect
                            std::thread::sleep(std::time::Duration::from_secs(2));
                        }
                        // Continue with next batch instead of failing completely
                    }
                }

                // Small delay between batches to be nice to the server
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // Retry failed UIDs one by one
            // Some emails may have malformed BODYSTRUCTURE that causes batch fetch to fail
            // Individual retry allows us to:
            // 1. Successfully fetch most emails that just had connection issues
            // 2. Mark truly problematic emails as "no attachments" to prevent infinite retry
            if !failed_uids.is_empty() {
                println!(
                    "🔄 Retrying {} failed emails individually...",
                    failed_uids.len()
                );

                for uid in failed_uids {
                    // Check if task was cancelled
                    if cancel_token.load(Ordering::Relaxed) {
                        println!("⏹️ Background BODYSTRUCTURE fetch cancelled during retry");
                        break;
                    }

                    match imap_session.uid_fetch(uid.to_string(), "(UID BODYSTRUCTURE)") {
                        Ok(messages) => {
                            for msg in messages.iter() {
                                if let Some(uid) = msg.uid {
                                    let (has_attachments, attachment_count) =
                                        check_attachments(msg);
                                    results.push((uid, has_attachments, attachment_count));
                                }
                            }
                        }
                        Err(e) => {
                            let error_str = format!("{:?}", e);
                            eprintln!("  ⚠️ UID {} still failed: {}", uid, e);

                            // If connection error during individual retry, try to reconnect
                            if error_str.contains("Bye")
                                || error_str.contains("TagMismatch")
                                || error_str.contains("Connection")
                            {
                                eprintln!(
                                    "  🔌 Connection lost during individual retry, reconnecting..."
                                );
                                match reconnect(&config_clone, &folder_name_clone) {
                                    Ok(new_session) => {
                                        imap_session = new_session;
                                        println!("  ✅ Reconnected, continuing with next email");
                                    }
                                    Err(e) => {
                                        eprintln!("  ❌ Failed to reconnect: {}", e);
                                        break; // Stop retrying if we can't reconnect
                                    }
                                }
                                std::thread::sleep(std::time::Duration::from_secs(1));
                            } else {
                                // Non-connection error (e.g., malformed BODYSTRUCTURE): mark as "no attachments"
                                // This prevents the email from being stuck in retry loop forever
                                // Better to show "no attachments" than never complete the background task
                                println!(
                                    "  ℹ️ Marking UID {} as no attachments due to persistent error",
                                    uid
                                );
                                results.push((uid as u32, false, 0));
                            }
                        }
                    }

                    // Small delay between individual fetches
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }

            Ok(results)
        })
        .await
        .map_err(|e| format!("Background task panicked: {}", e))??;

    // Update database with results
    let mut updated_count = 0;
    for (uid, has_attachments, _attachment_count) in results {
        let result = sqlx::query(
            "UPDATE emails SET has_attachments = ? 
             WHERE account_id = ? AND folder_name = ? AND uid = ?",
        )
        .bind(has_attachments as i64)
        .bind(account_id)
        .bind(&folder_name)
        .bind(uid as i64)
        .execute(pool.as_ref())
        .await;

        match result {
            Ok(_) => updated_count += 1,
            Err(e) => eprintln!("⚠️ Failed to update attachment info for UID {}: {}", uid, e),
        }
    }

    println!(
        "✅ Background BODYSTRUCTURE fetch complete: {} emails updated",
        updated_count
    );
    Ok(())
}

/// Reconnect to IMAP server and select folder
fn reconnect(
    config: &AccountConfig,
    folder_name: &str,
) -> Result<imap::Session<Box<dyn imap::ImapConnection>>, String> {
    // Connect to IMAP
    let mut imap_session = imap_helpers::connect_and_login(config)
        .map_err(|e| format!("Failed to connect to IMAP: {}", e))?;

    // Select folder
    imap_session
        .select(folder_name)
        .map_err(|e| format!("Failed to select folder '{}': {}", folder_name, e))?;

    Ok(imap_session)
}

/// Check if a message has attachments by examining its BODYSTRUCTURE
fn check_attachments(msg: &imap::types::Fetch) -> (bool, usize) {
    if let Some(body_structure) = msg.bodystructure() {
        // Use debug string to detect attachments (same as codec.rs approach)
        let debug_str = format!("{:?}", body_structure);
        let lower = debug_str.to_lowercase();

        // Check for attachment indicators
        let has_attachments = lower.contains("attachment") || lower.contains("filename");

        if has_attachments {
            // Count occurrences of "attachment" or "filename" as rough estimate
            let attachment_count =
                lower.matches("attachment").count() + lower.matches("filename").count();
            // Divide by 2 because each attachment might have both markers
            let estimated_count = (attachment_count / 2).max(1);
            (true, estimated_count)
        } else {
            (false, 0)
        }
    } else {
        (false, 0)
    }
}
