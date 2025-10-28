// Email synchronization logic
// This module handles incremental sync using UIDVALIDITY and UIDs

use crate::commands::emails::cache::{load_emails_from_cache, save_emails_to_cache};
use crate::commands::emails::codec::{
    check_for_attachments, decode_bytes_to_string, decode_header, parse_email_date_with_fallback,
};
use crate::commands::emails::fetch_bodystructure;
use crate::commands::emails::imap_helpers;
use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::{AccountConfig, EmailHeader};
use chrono::Utc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::command;

/// Struct to hold sync state from database
#[derive(Clone)]
struct SyncState {
    uidvalidity: Option<i64>,
    highest_uid: Option<i64>,
}

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
    let emails = incremental_sync(config.clone(), account_id, &folder_name).await?;

    println!(
        "✅ Incremental sync completed: {} emails in cache",
        emails.len()
    );

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

/// Perform incremental synchronization using UIDVALIDITY and UIDs
async fn incremental_sync(
    config: AccountConfig,
    account_id: i32,
    folder_name: &str,
) -> Result<Vec<EmailHeader>, String> {
    // Ensure we have a valid access token
    let config = ensure_valid_token(config).await?;

    // Get cached sync state (UIDVALIDITY and highest UID)
    let sync_state = get_sync_state(account_id, folder_name).await?;
    let sync_state_for_task = sync_state.clone();
    let folder_name_owned = folder_name.to_string();
    let config_for_uid_check = config.clone();

    // Debug: Check what's actually in the cache
    let pool = db::pool();
    let cache_max_uid = sqlx::query_as::<_, (Option<i64>,)>(
        "SELECT MAX(uid) FROM emails WHERE account_id = ? AND folder_name = ?",
    )
    .bind(account_id)
    .bind(folder_name)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to query cache max UID: {}", e))?
    .0;

    println!(
        "🔍 Cache state: highest_uid in sync_status = {:?}, MAX(uid) in emails table = {:?}",
        sync_state.as_ref().and_then(|s| s.highest_uid),
        cache_max_uid
    );

    // Connect to IMAP and check current state
    let (server_uidvalidity, _server_exists, new_emails) =
        tokio::task::spawn_blocking(move || -> Result<(u32, u32, Vec<EmailHeader>), String> {
            // Use new imap_helpers to connect and login
            let mut imap_session = imap_helpers::connect_and_login(&config)?;

            println!("✅ IMAP authentication successful");

            // SELECT the folder and get UIDVALIDITY
            let mailbox = imap_session
                .select(&folder_name_owned)
                .map_err(|e| format!("Cannot access folder '{}': {}", folder_name_owned, e))?;

            let server_uidvalidity = mailbox.uid_validity.unwrap_or(0);
            let server_exists = mailbox.exists;

            println!(
                "📊 Server state: UIDVALIDITY={}, EXISTS={}",
                server_uidvalidity, server_exists
            );

            // Determine sync strategy based on UIDVALIDITY
            let new_emails = if let Some(ref sync_state) = sync_state_for_task {
                if sync_state.uidvalidity != Some(server_uidvalidity as i64) {
                    // Full sync needed: UIDVALIDITY changed
                    println!("⚠️ UIDVALIDITY changed! Full resync required.");

                    // Fetch all emails in the mailbox in batches
                    if server_exists == 0 {
                        Vec::new()
                    } else {
                        // Fetch in batches to avoid overwhelming the IMAP server and parser
                        // Start with batch size 20, increase exponentially until hitting server limit
                        let mut batch_size = 20u32;
                        let mut max_batch_size: Option<u32> = None; // Lock batch size after first Bye error
                        let mut all_headers = Vec::new();
                        let mut current_pos = 1u32;

                        println!(
                            "📥 Full resync: fetching all {} messages (starting batch size: {})",
                            server_exists, batch_size
                        );

                        let mut batch_num = 0u32;
                        while current_pos <= server_exists {
                            batch_num += 1;
                            let end_seq = (current_pos + batch_size - 1).min(server_exists);
                            let seq_range = format!("{}:{}", current_pos, end_seq);
                            let count = end_seq - current_pos + 1;

                            println!(
                                "  📦 Batch {}: fetching messages {} ({} messages)",
                                batch_num,
                                seq_range,
                                count
                            );

                            // Fetch without BODYSTRUCTURE (causes issues with GMX)
                            match imap_session.fetch(seq_range.as_str(), "(UID ENVELOPE FLAGS INTERNALDATE RFC822.SIZE)") {
                                Ok(messages) => {
                                    let batch_headers = parse_email_headers(messages.iter());
                                    all_headers.extend(batch_headers);

                                    println!(
                                        "  ✓ Batch {} complete, {} total emails so far",
                                        batch_num,
                                        all_headers.len()
                                    );

                                    current_pos = end_seq + 1;

                                    // Gradually increase batch size if successful
                                    // If max_batch_size is set (after a Bye error), respect that limit
                                    if let Some(max) = max_batch_size {
                                        if batch_size < max {
                                            batch_size = (batch_size * 2).min(max);
                                            println!("  📈 Increasing batch size to {} (locked max: {})", batch_size, max);
                                        }
                                    } else {
                                        // No limit yet, keep doubling
                                        batch_size *= 2;
                                        println!("  📈 Increasing batch size to {}", batch_size);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ IMAP FETCH failed for batch {}", batch_num);
                                    eprintln!("   Range: {}", seq_range);
                                    eprintln!("   Batch size: {}", batch_size);
                                    eprintln!("   Error details: {:?}", e);

                                    // Check if this is a connection error (Bye)
                                    if is_connection_error(&e) {
                                        eprintln!("  🔌 Connection lost (Bye error), attempting to reconnect...");

                                        // Lock to the last successful batch size (before this failed attempt)
                                        let last_successful_size = (batch_size / 2).max(10);
                                        if max_batch_size.is_none() {
                                            max_batch_size = Some(last_successful_size);
                                            println!("  🔒 Locking batch size to last successful: {}", last_successful_size);
                                        }

                                        // Try to logout the old session gracefully (ignore errors)
                                        let _ = imap_session.logout();

                                        // Wait 2 seconds before reconnecting
                                        println!("  ⏱️ Waiting 2 seconds before reconnecting...");
                                        std::thread::sleep(std::time::Duration::from_secs(2));

                                        // Reconnect to IMAP server
                                        match imap_helpers::connect_and_login(&config) {
                                            Ok(new_session) => {
                                                imap_session = new_session;
                                                println!("  ✅ Reconnected successfully");

                                                // Re-select the folder
                                                match imap_session.select(&folder_name_owned) {
                                                    Ok(_) => {
                                                        println!("  ✅ Folder re-selected");
                                                        // Use the locked batch size
                                                        batch_size = last_successful_size;
                                                        println!("  ⚠️ Retrying with safe batch size: {}", batch_size);
                                                        // Don't increment current_pos, we'll retry this range
                                                    }
                                                    Err(select_err) => {
                                                        return Err(format!("Failed to re-select folder after reconnection: {}", select_err));
                                                    }
                                                }
                                            }
                                            Err(conn_err) => {
                                                return Err(format!("Failed to reconnect after Bye error: {}", conn_err));
                                            }
                                        }
                                    } else {
                                        // Not a connection error, just reduce batch size

                                        // If batch size is already very small, give up
                                        if batch_size <= 10 {
                                            return Err(format!("Failed to fetch batch {} even with minimum batch size: {}", batch_num, e));
                                        }

                                        // Reduce batch size and retry
                                        batch_size = (batch_size / 2).max(10);
                                        println!("  ⚠️ Retrying with smaller batch size: {}", batch_size);
                                        // Don't increment current_pos, we'll retry this range
                                    }
                                }
                            }
                        }

                        // Sort by timestamp descending (newest first)
                        all_headers.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                        println!("✅ Full resync complete: fetched {} emails in {} batch(es)", all_headers.len(), batch_num);
                        all_headers
                    }
                } else {
                    // Incremental sync: fetch only new messages
                    let highest_uid = sync_state.highest_uid.unwrap_or(0);

                    println!("🔄 Incremental sync from UID > {}", highest_uid);

                    if highest_uid == 0 || server_exists == 0 {
                        // No previous emails or empty folder
                        Vec::new()
                    } else {
                        // First, use UID SEARCH to find if there are any new messages
                        // This avoids Gmail's bug where UID FETCH with reversed range returns old messages
                        let search_criteria = format!("UID {}:*", highest_uid + 1);

                        println!("🔍 Searching for new messages: {}", search_criteria);

                        let search_result = match imap_session.uid_search(&search_criteria) {
                            Ok(uids) => {
                                // Convert HashSet to Vec and sort
                                let mut uid_vec: Vec<u32> = uids.into_iter().collect();
                                uid_vec.sort_unstable();
                                uid_vec
                            }
                            Err(e) => {
                                eprintln!("⚠️ UID SEARCH failed: {}, falling back to FETCH", e);
                                Vec::new()
                            }
                        };

                        if search_result.is_empty() {
                            println!("✅ No new messages (SEARCH returned empty)");
                            Vec::new()
                        } else {
                            println!("📋 SEARCH found {} UID(s): {:?}", search_result.len(), search_result);

                            // Filter out UIDs <= highest_uid (Gmail bug workaround)
                            let new_uids: Vec<u32> = search_result
                                .into_iter()
                                .filter(|&uid| uid > highest_uid as u32)
                                .collect();

                            if new_uids.is_empty() {
                                println!("✅ No genuinely new messages after filtering");
                                Vec::new()
                            } else {
                                println!("📥 Fetching {} new message(s) with UIDs: {:?}", new_uids.len(), new_uids);

                                // Start with batch size 20, increase exponentially until hitting server limit
                                let mut batch_size = 20usize;
                                let mut max_batch_size: Option<usize> = None; // Lock batch size after first Bye error
                                let mut all_new_headers = Vec::new();
                                let mut current_idx = 0usize;
                                let total_count = new_uids.len();

                                let mut batch_num = 0usize;
                                while current_idx < total_count {
                                    batch_num += 1;
                                    let end_idx = (current_idx + batch_size).min(total_count);
                                    let uid_chunk = &new_uids[current_idx..end_idx];
                                    let chunk_size = uid_chunk.len();

                                    println!("  📦 Batch {}: fetching {} message(s)", batch_num, chunk_size);

                                    // Build UID list for FETCH
                                    let uid_list = uid_chunk
                                        .iter()
                                        .map(|uid| uid.to_string())
                                        .collect::<Vec<_>>()
                                        .join(",");

                                    // Fetch without BODYSTRUCTURE (causes issues with GMX)
                                    match imap_session.uid_fetch(&uid_list, "(UID ENVELOPE FLAGS INTERNALDATE RFC822.SIZE)") {
                                        Ok(messages) => {
                                            let count = messages.len();
                                            if count > 0 {
                                                println!("  ✨ Batch {} found {} raw message(s) from IMAP", batch_num, count);

                                                // Debug: Log the actual UIDs returned by IMAP
                                                for (idx, msg) in messages.iter().enumerate() {
                                                    println!("    📋 Message {}: UID = {:?}", idx + 1, msg.uid);
                                                }

                                                let parsed = parse_email_headers(messages.iter().rev());

                                                // Debug: Log the parsed UIDs
                                                println!("    📝 Parsed UIDs: {:?}", parsed.iter().map(|e| e.uid).collect::<Vec<_>>());

                                                // Filter out emails with UID <= highest_uid
                                                // This handles cases where IMAP server returns UIDs we already have
                                                let filtered: Vec<EmailHeader> = parsed
                                                    .into_iter()
                                                    .filter(|email| email.uid > highest_uid as u32)
                                                    .collect();

                                                if filtered.len() < count {
                                                    println!(
                                                        "    🔍 Filtered out {} duplicate/old email(s), keeping {} new",
                                                        count - filtered.len(),
                                                        filtered.len()
                                                    );
                                                }

                                                all_new_headers.extend(filtered);
                                            } else {
                                                println!("  ✅ Batch {} returned no messages", batch_num);
                                            }

                                            println!("  ✓ Batch {} complete, {} total new emails so far", batch_num, all_new_headers.len());

                                            // Move to next batch
                                            current_idx = end_idx;

                                            // Gradually increase batch size if successful
                                            // If max_batch_size is set (after a Bye error), respect that limit
                                            if let Some(max) = max_batch_size {
                                                if batch_size < max {
                                                    batch_size = (batch_size * 2).min(max);
                                                    println!("  📈 Increasing batch size to {} (locked max: {})", batch_size, max);
                                                }
                                            } else {
                                                // No limit yet, keep doubling
                                                batch_size *= 2;
                                                println!("  📈 Increasing batch size to {}", batch_size);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("  ⚠️ Batch {} failed to fetch: {}", batch_num, e);

                                            // Check if this is a connection error (Bye)
                                            if is_connection_error(&e) {
                                                // Lock to the last successful batch size
                                                let last_successful_size = (batch_size / 2).max(10);
                                                if max_batch_size.is_none() {
                                                    max_batch_size = Some(last_successful_size);
                                                    println!("  🔒 Locking batch size to last successful: {}", last_successful_size);
                                                }
                                                batch_size = last_successful_size;
                                                println!("  ⚠️ Retrying with safe batch size: {}", batch_size);
                                                // Don't advance current_idx, retry this batch
                                            } else {
                                                // Not a Bye error, reduce batch size and retry
                                                if batch_size > 10 {
                                                    batch_size = (batch_size / 2).max(10);
                                                    println!("  ⚠️ Retrying with smaller batch size: {}", batch_size);
                                                } else {
                                                    // If already at minimum, skip this batch
                                                    eprintln!("  ❌ Skipping batch {} (already at minimum batch size)", batch_num);
                                                    current_idx = end_idx;
                                                }
                                            }
                                        }
                                    }
                                }

                                if all_new_headers.is_empty() {
                                    println!("✅ No new messages after fetching all batches");
                                } else {
                                    println!("✨ Total {} genuinely new message(s) from all batches", all_new_headers.len());
                                }

                                all_new_headers
                            }
                        }
                    }
                }
            } else {
                // Full sync needed: no previous state
                println!("🆕 First sync for this folder.");

                // Fetch all emails in the mailbox
                if server_exists == 0 {
                    Vec::new()
                } else {
                    // Fetch in batches to avoid overwhelming the IMAP server and parser
                    // Start with batch size 20, increase exponentially until hitting server limit
                    let mut batch_size = 20u32;
                    let mut max_batch_size: Option<u32> = None; // Lock batch size after first Bye error
                    let mut all_headers = Vec::new();
                    let mut current_pos = 1u32;

                    println!(
                        "📥 Initial sync: fetching all {} messages (starting batch size: {})",
                        server_exists, batch_size
                    );

                    let mut batch_num = 0u32;
                    while current_pos <= server_exists {
                        batch_num += 1;
                        let end_seq = (current_pos + batch_size - 1).min(server_exists);
                        let seq_range = format!("{}:{}", current_pos, end_seq);
                        let count = end_seq - current_pos + 1;

                        println!(
                            "  📦 Batch {}: fetching messages {} ({} messages)",
                            batch_num,
                            seq_range,
                            count
                        );

                        // Fetch without BODYSTRUCTURE (causes issues with GMX)
                        match imap_session.fetch(seq_range.as_str(), "(UID ENVELOPE FLAGS INTERNALDATE RFC822.SIZE)") {
                            Ok(messages) => {
                                let batch_headers = parse_email_headers(messages.iter());
                                all_headers.extend(batch_headers);

                                println!(
                                    "  ✓ Batch {} complete, {} total emails so far",
                                    batch_num,
                                    all_headers.len()
                                );

                                current_pos = end_seq + 1;

                                // Gradually increase batch size if successful
                                // If max_batch_size is set (after a Bye error), respect that limit
                                if let Some(max) = max_batch_size {
                                    if batch_size < max {
                                        batch_size = (batch_size * 2).min(max);
                                        println!("  📈 Increasing batch size to {} (locked max: {})", batch_size, max);
                                    }
                                } else {
                                    // No limit yet, keep doubling
                                    batch_size *= 2;
                                    println!("  📈 Increasing batch size to {}", batch_size);
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ IMAP FETCH failed for batch {}", batch_num);
                                eprintln!("   Range: {}", seq_range);
                                eprintln!("   Batch size: {}", batch_size);
                                eprintln!("   Error details: {:?}", e);

                                // Check if this is a connection error (Bye)
                                if is_connection_error(&e) {
                                    eprintln!("  🔌 Connection lost (Bye error), attempting to reconnect...");

                                    // Lock to the last successful batch size (before this failed attempt)
                                    let last_successful_size = (batch_size / 2).max(10);
                                    if max_batch_size.is_none() {
                                        max_batch_size = Some(last_successful_size);
                                        println!("  🔒 Locking batch size to last successful: {}", last_successful_size);
                                    }

                                    // Try to logout the old session gracefully (ignore errors)
                                    let _ = imap_session.logout();

                                    // Wait 2 seconds before reconnecting
                                    println!("  ⏱️ Waiting 2 seconds before reconnecting...");
                                    std::thread::sleep(std::time::Duration::from_secs(2));

                                    // Reconnect to IMAP server
                                    match imap_helpers::connect_and_login(&config) {
                                        Ok(new_session) => {
                                            imap_session = new_session;
                                            println!("  ✅ Reconnected successfully");

                                            // Re-select the folder
                                            match imap_session.select(&folder_name_owned) {
                                                Ok(_) => {
                                                    println!("  ✅ Folder re-selected");
                                                    // Use the locked batch size
                                                    batch_size = last_successful_size;
                                                    println!("  ⚠️ Retrying with safe batch size: {}", batch_size);
                                                    // Don't increment current_pos, we'll retry this range
                                                }
                                                Err(select_err) => {
                                                    return Err(format!("Failed to re-select folder after reconnection: {}", select_err));
                                                }
                                            }
                                        }
                                        Err(conn_err) => {
                                            return Err(format!("Failed to reconnect after Bye error: {}", conn_err));
                                        }
                                    }
                                } else {
                                    // Not a connection error, just reduce batch size

                                    // If batch size is already very small, give up
                                    if batch_size <= 10 {
                                        return Err(format!("Failed to fetch batch {} even with minimum batch size: {}", batch_num, e));
                                    }

                                    // Reduce batch size and retry
                                    batch_size = (batch_size / 2).max(10);
                                    println!("  ⚠️ Retrying with smaller batch size: {}", batch_size);
                                    // Don't increment current_pos, we'll retry this range
                                }
                            }
                        }
                    }

                    // Sort by timestamp descending (newest first)
                    all_headers.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                    println!("✅ Initial sync complete: fetched {} emails in {} batch(es)", all_headers.len(), batch_num);
                    all_headers
                }
            };

            let _ = imap_session.logout();
            Ok((server_uidvalidity, server_exists, new_emails))
        })
        .await
        .map_err(|e| e.to_string())??;

    println!("✅ Fetched {} new email(s) from server", new_emails.len());

    // Save new emails to cache
    if !new_emails.is_empty() {
        save_emails_to_cache(account_id, folder_name, &new_emails).await?;
    }

    // Get all UIDs currently on server to detect deletions
    println!("🔍 Checking for deleted emails...");
    let server_uids = get_all_server_uids(config_for_uid_check, folder_name).await?;

    // Delete emails from cache that no longer exist on server
    let deleted_count =
        delete_missing_emails_from_cache(account_id, folder_name, &server_uids).await?;
    if deleted_count > 0 {
        println!("🗑️ Removed {} deleted email(s) from cache", deleted_count);
    }

    // Update sync state with new UIDVALIDITY and highest UID
    // IMPORTANT: Only update highest_uid based on emails we actually fetched and saved
    // Do NOT use server_max_uid because there may be a time gap between:
    // 1. Fetching new emails (first IMAP connection)
    // 2. Querying all server UIDs (second IMAP connection)
    // If a new email arrives in between, server_max_uid will be higher than what we actually got

    let new_emails_max_uid = new_emails.iter().map(|e| e.uid).max();
    let server_max_uid = server_uids.iter().copied().max();
    let previous_highest_uid = sync_state
        .as_ref()
        .and_then(|s| s.highest_uid.map(|u| u as u32));

    // Use the maximum of:
    // 1. Actually fetched emails (safe - we have the content)
    // 2. Previous highest_uid (safe - we had it before)
    // Do NOT use server_max_uid to avoid the race condition
    let new_highest_uid = new_emails_max_uid.max(previous_highest_uid).unwrap_or(0);

    println!(
        "📌 Updating highest UID: {} (new_emails_max={:?}, server_max={:?}, prev={:?}, server_total={})",
        new_highest_uid,
        new_emails_max_uid,
        server_max_uid,
        previous_highest_uid,
        server_uids.len()
    );

    update_sync_state(
        account_id,
        folder_name,
        server_uidvalidity as i64,
        new_highest_uid as i64,
    )
    .await?;

    // Return all cached emails (for display)
    load_emails_from_cache(account_id, Some(folder_name.to_string())).await
}

/// Check if an IMAP error is a connection error (Bye) that requires reconnection
fn is_connection_error(error: &imap::Error) -> bool {
    matches!(error, imap::Error::Bye(_))
}

/// Helper function to parse IMAP fetch results into EmailHeader
/// In imap 3.0.0, Fetch type requires lifetime parameter
fn parse_email_headers<'a, I>(messages: I) -> Vec<EmailHeader>
where
    I: Iterator<Item = &'a imap::types::Fetch<'a>>,
{
    let mut headers = Vec::new();

    for msg in messages {
        let envelope = match msg.envelope() {
            Some(e) => e,
            None => continue,
        };

        let subject = envelope
            .subject
            .as_ref()
            .map(|s| {
                // In imap 3.0.0, envelope fields are Cow<[u8]> instead of &[u8]
                let raw_subject = decode_bytes_to_string(s.as_ref());
                decode_header(&raw_subject)
            })
            .unwrap_or_else(|| "(No Subject)".to_string());

        let from = envelope
            .from
            .as_ref()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        if let Some(ref name_bytes) = addr.name {
                            let name = decode_bytes_to_string(name_bytes.as_ref());
                            if !name.trim().is_empty() {
                                return decode_header(&name);
                            }
                        }
                        let mailbox = decode_bytes_to_string(
                            addr.mailbox.clone().unwrap_or_default().as_ref(),
                        );
                        let host =
                            decode_bytes_to_string(addr.host.clone().unwrap_or_default().as_ref());
                        format!("{}@{}", mailbox, host)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "(Unknown Sender)".to_string());

        let date = envelope
            .date
            .as_ref()
            .map(|d| decode_bytes_to_string(d.as_ref()))
            .unwrap_or_else(|| "(No Date)".to_string());

        // Get INTERNALDATE as a fallback for date parsing
        let internal_date = msg
            .internal_date()
            .map(|d| format!("{}", d.format("%a, %d %b %Y %H:%M:%S %z")));

        // Use INTERNALDATE as fallback if Date header parsing fails
        let timestamp = parse_email_date_with_fallback(&date, internal_date.as_deref());

        let to = envelope
            .to
            .as_ref()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        if let Some(ref name_bytes) = addr.name {
                            let name = decode_bytes_to_string(name_bytes.as_ref());
                            if !name.trim().is_empty() {
                                return decode_header(&name);
                            }
                        }
                        let mailbox = decode_bytes_to_string(
                            addr.mailbox.clone().unwrap_or_default().as_ref(),
                        );
                        let host =
                            decode_bytes_to_string(addr.host.clone().unwrap_or_default().as_ref());
                        format!("{}@{}", mailbox, host)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "(Unknown Recipient)".to_string());

        let cc = envelope
            .cc
            .as_ref()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        if let Some(ref name_bytes) = addr.name {
                            let name = decode_bytes_to_string(name_bytes.as_ref());
                            if !name.trim().is_empty() {
                                return decode_header(&name);
                            }
                        }
                        let mailbox = decode_bytes_to_string(
                            addr.mailbox.clone().unwrap_or_default().as_ref(),
                        );
                        let host =
                            decode_bytes_to_string(addr.host.clone().unwrap_or_default().as_ref());
                        format!("{}@{}", mailbox, host)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "".to_string());

        // Check if email has attachments by examining BODYSTRUCTURE
        let has_attachments = msg
            .bodystructure()
            .map(check_for_attachments)
            .unwrap_or(false);

        // Check if email has been read by examining FLAGS
        let seen = msg
            .flags()
            .iter()
            .any(|flag| matches!(flag, imap::types::Flag::Seen));

        headers.push(EmailHeader {
            uid: msg.uid.unwrap_or(0),
            subject,
            from,
            to,
            cc,
            date,
            timestamp,
            has_attachments,
            seen,
        });
    }

    // Note: Sorting is now done by the caller
    headers
}

/// Get sync state for a folder
async fn get_sync_state(account_id: i32, folder_name: &str) -> Result<Option<SyncState>, String> {
    let pool = db::pool();

    let result = sqlx::query_as::<_, (Option<i64>, Option<i64>)>(
        "SELECT uidvalidity, highest_uid FROM sync_status WHERE account_id = ? AND folder_name = ?",
    )
    .bind(account_id)
    .bind(folder_name)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to get sync state: {}", e))?;

    Ok(result.map(|(uidvalidity, highest_uid)| SyncState {
        uidvalidity,
        highest_uid,
    }))
}

/// Update sync state for a folder
async fn update_sync_state(
    account_id: i32,
    folder_name: &str,
    uidvalidity: i64,
    highest_uid: i64,
) -> Result<(), String> {
    let pool = db::pool();
    let current_time = Utc::now().timestamp();

    sqlx::query(
        "INSERT OR REPLACE INTO sync_status (account_id, folder_name, last_sync_time, uidvalidity, highest_uid)
        VALUES (?, ?, ?, ?, ?)",
    )
    .bind(account_id)
    .bind(folder_name)
    .bind(current_time)
    .bind(uidvalidity)
    .bind(highest_uid)
    .execute(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to update sync state: {}", e))?;

    println!(
        "✅ Updated sync state: UIDVALIDITY={}, highest_uid={}",
        uidvalidity, highest_uid
    );

    Ok(())
}

/// Get all UIDs currently on server
async fn get_all_server_uids(config: AccountConfig, folder_name: &str) -> Result<Vec<u32>, String> {
    let config = ensure_valid_token(config).await?;
    let folder_name_owned = folder_name.to_string();

    tokio::task::spawn_blocking(move || -> Result<Vec<u32>, String> {
        // Use new imap_helpers to connect and login
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

        // SELECT the folder
        imap_session
            .select(&folder_name_owned)
            .map_err(|e| format!("Cannot select folder: {}", e))?;

        // Search for all messages to get UIDs
        let uid_results = imap_session
            .uid_search("ALL")
            .map_err(|e| format!("Failed to search UIDs: {}", e))?;

        let uids: Vec<u32> = uid_results.iter().copied().collect();

        let _ = imap_session.logout();
        Ok(uids)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Delete emails from cache that no longer exist on server
async fn delete_missing_emails_from_cache(
    account_id: i32,
    folder_name: &str,
    server_uids: &[u32],
) -> Result<u64, String> {
    let pool = db::pool();

    // Get all cached UIDs for this folder
    let cached_uids: Vec<u32> = sqlx::query_as::<_, (i64,)>(
        "SELECT uid FROM emails WHERE account_id = ? AND folder_name = ?",
    )
    .bind(account_id)
    .bind(folder_name)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to get cached UIDs: {}", e))?
    .into_iter()
    .map(|(uid,)| uid as u32)
    .collect();

    // Find UIDs that exist in cache but not on server
    let uids_to_delete: Vec<u32> = cached_uids
        .into_iter()
        .filter(|uid| !server_uids.contains(uid))
        .collect();

    if uids_to_delete.is_empty() {
        return Ok(0);
    }

    println!("🗑️ Deleting UIDs from cache: {:?}", uids_to_delete);

    // Delete emails with these UIDs
    let mut deleted_count = 0u64;
    for uid in uids_to_delete {
        let result =
            sqlx::query("DELETE FROM emails WHERE account_id = ? AND folder_name = ? AND uid = ?")
                .bind(account_id)
                .bind(folder_name)
                .bind(uid as i64)
                .execute(pool.as_ref())
                .await
                .map_err(|e| format!("Failed to delete email UID {}: {}", uid, e))?;

        deleted_count += result.rows_affected();
    }

    Ok(deleted_count)
}

/// Get last sync time for a folder
#[command]
pub async fn get_last_sync_time(account_id: i32, folder: Option<String>) -> Result<i64, String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    let pool = db::pool();

    let result = sqlx::query_as::<_, (i64,)>(
        "SELECT last_sync_time FROM sync_status WHERE account_id = ? AND folder_name = ?",
    )
    .bind(account_id)
    .bind(&folder_name)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to get last sync time: {}", e))?;

    Ok(result.map(|(time,)| time).unwrap_or(0))
}

/// Check if sync is needed based on interval
#[command]
pub async fn should_sync(
    account_id: i32,
    folder: Option<String>,
    sync_interval: i64,
) -> Result<bool, String> {
    // sync_interval in seconds, 0 = manual, -1 = never
    if sync_interval == -1 {
        return Ok(false); // Never sync
    }
    if sync_interval == 0 {
        return Ok(false); // Manual only
    }

    let last_sync = get_last_sync_time(account_id, folder).await?;
    let current_time = Utc::now().timestamp();
    let elapsed = current_time - last_sync;

    Ok(elapsed >= sync_interval)
}
