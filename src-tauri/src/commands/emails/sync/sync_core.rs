// Core synchronization logic
// This module handles the incremental sync algorithm and deletion detection

use super::sync_fetch::{fetch_all_by_sequence, fetch_new_by_uid_list};
use super::sync_state::{get_sync_state, update_sync_state};
use crate::commands::emails::cache::save_emails_to_cache;
use crate::commands::emails::imap_helpers;
use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::{AccountConfig, EmailHeader};

/// Perform incremental synchronization using UIDVALIDITY and UIDs
pub async fn incremental_sync(
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

                    fetch_all_by_sequence(
                        &mut imap_session,
                        server_exists,
                        &config,
                        &folder_name_owned,
                    )?
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
                            println!(
                                "📋 SEARCH found {} UID(s): {:?}",
                                search_result.len(),
                                search_result
                            );

                            // Filter out UIDs <= highest_uid (Gmail bug workaround)
                            let new_uids: Vec<u32> = search_result
                                .into_iter()
                                .filter(|&uid| uid > highest_uid as u32)
                                .collect();

                            if new_uids.is_empty() {
                                println!("✅ No genuinely new messages after filtering");
                                Vec::new()
                            } else {
                                fetch_new_by_uid_list(&mut imap_session, new_uids, highest_uid)?
                            }
                        }
                    }
                }
            } else {
                // Full sync needed: no previous state
                println!("🆕 First sync for this folder.");

                fetch_all_by_sequence(
                    &mut imap_session,
                    server_exists,
                    &config,
                    &folder_name_owned,
                )?
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

    Ok(new_emails)
}

/// Get all UIDs currently on server
pub async fn get_all_server_uids(
    config: AccountConfig,
    folder_name: &str,
) -> Result<Vec<u32>, String> {
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
pub async fn delete_missing_emails_from_cache(
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
