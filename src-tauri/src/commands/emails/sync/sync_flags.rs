// Email flag synchronization
// This module handles syncing read/starred flags between server and cache

use crate::commands::emails::imap_helpers;
use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::AccountConfig;
use tauri::command;

/// Sync flags (seen, flagged) for existing emails in cache
/// This ensures local cache stays in sync with server when flags change from other clients
/// This is called when IDLE detects flag changes, not during regular incremental sync
#[command]
pub async fn sync_email_flags(
    account_id: i32,
    folder_name: &str,
    config: AccountConfig,
) -> Result<(), String> {
    let start_time = std::time::Instant::now();
    let config = ensure_valid_token(config).await?;

    // Get all cached email UIDs
    let pool = db::pool();
    let cached_uids: Vec<u32> = sqlx::query_as::<_, (i64,)>(
        "SELECT uid FROM emails WHERE account_id = ? AND folder_name = ? ORDER BY uid ASC",
    )
    .bind(account_id)
    .bind(folder_name)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to get cached UIDs: {}", e))?
    .into_iter()
    .map(|(uid,)| uid as u32)
    .collect();

    if cached_uids.is_empty() {
        return Ok(());
    }

    let email_count = cached_uids.len();
    println!("🔄 Syncing flags for {} cached emails", email_count);
    let fetch_start = std::time::Instant::now();

    let folder_name_owned = folder_name.to_string();

    // Fetch flags from server in batches
    let flags_data =
        tokio::task::spawn_blocking(move || -> Result<Vec<(u32, bool, bool)>, String> {
            let mut imap_session = imap_helpers::connect_and_login(&config)?;

            imap_session
                .select(&folder_name_owned)
                .map_err(|e| format!("Cannot select folder: {}", e))?;

            let mut all_flags = Vec::new();

            // Process in batches of 100 to avoid overwhelming the server
            for chunk in cached_uids.chunks(100) {
                let uid_list = chunk
                    .iter()
                    .map(|uid| uid.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                match imap_session.uid_fetch(&uid_list, "(UID FLAGS)") {
                    Ok(messages) => {
                        for msg in messages.iter() {
                            let uid = msg.uid.unwrap_or(0);
                            let seen = msg
                                .flags()
                                .iter()
                                .any(|flag| matches!(flag, imap::types::Flag::Seen));
                            let flagged = msg
                                .flags()
                                .iter()
                                .any(|flag| matches!(flag, imap::types::Flag::Flagged));
                            all_flags.push((uid, seen, flagged));
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to fetch flags for batch: {}", e);
                        // Continue with other batches
                    }
                }
            }

            let _ = imap_session.logout();
            Ok(all_flags)
        })
        .await
        .map_err(|e| e.to_string())??;

    let fetch_elapsed = fetch_start.elapsed();
    println!(
        "⏱️ IMAP fetch took {:.2}s for {} emails",
        fetch_elapsed.as_secs_f64(),
        email_count
    );

    // Update cache with new flag values
    let db_start = std::time::Instant::now();
    let pool = db::pool();
    let mut updated_count = 0;
    let mut changed_count = 0;

    for (uid, seen, flagged) in flags_data {
        let result = sqlx::query(
            "UPDATE emails SET seen = ?, flagged = ? WHERE account_id = ? AND folder_name = ? AND uid = ? AND (seen != ? OR flagged != ?)"
        )
        .bind(seen as i64)
        .bind(flagged as i64)
        .bind(account_id)
        .bind(folder_name)
        .bind(uid as i64)
        .bind(seen as i64)
        .bind(flagged as i64)
        .execute(pool.as_ref())
        .await;

        if let Ok(r) = result {
            if r.rows_affected() > 0 {
                changed_count += 1;
            }
            updated_count += 1;
        }
    }

    let db_elapsed = db_start.elapsed();
    let total_elapsed = start_time.elapsed();

    println!(
        "⏱️ Database update took {:.2}s for {} emails",
        db_elapsed.as_secs_f64(),
        updated_count
    );
    println!(
        "✅ Flag sync complete: {}/{} emails changed, total time: {:.2}s (fetch: {:.2}s, db: {:.2}s)",
        changed_count,
        email_count,
        total_elapsed.as_secs_f64(),
        fetch_elapsed.as_secs_f64(),
        db_elapsed.as_secs_f64()
    );

    Ok(())
}

/// Sync flags for a specific email UID (efficient for FlagsChanged events)
/// This is called when IDLE detects a flag change for a specific message
#[command]
pub async fn sync_specific_email_flags(
    account_id: i32,
    folder_name: &str,
    uid: u32,
    config: AccountConfig,
) -> Result<(), String> {
    let start_time = std::time::Instant::now();
    let config = ensure_valid_token(config).await?;

    println!("🔄 Syncing flags for UID {} in folder {}", uid, folder_name);

    let folder_name_owned = folder_name.to_string();

    // Fetch flags from server for specific UID
    let flags_data = tokio::task::spawn_blocking(move || -> Result<(bool, bool), String> {
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

        imap_session
            .select(&folder_name_owned)
            .map_err(|e| format!("Cannot select folder: {}", e))?;

        // Fetch flags for this specific UID
        match imap_session.uid_fetch(uid.to_string(), "(UID FLAGS)") {
            Ok(messages) => {
                if let Some(msg) = messages.iter().next() {
                    let seen = msg
                        .flags()
                        .iter()
                        .any(|flag| matches!(flag, imap::types::Flag::Seen));
                    let flagged = msg
                        .flags()
                        .iter()
                        .any(|flag| matches!(flag, imap::types::Flag::Flagged));
                    let _ = imap_session.logout();
                    Ok((seen, flagged))
                } else {
                    let _ = imap_session.logout();
                    Err(format!("UID {} not found on server", uid))
                }
            }
            Err(e) => {
                let _ = imap_session.logout();
                Err(format!("Failed to fetch flags for UID {}: {}", uid, e))
            }
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let (seen, flagged) = flags_data;

    // Update cache
    let pool = db::pool();
    let result = sqlx::query(
        "UPDATE emails SET seen = ?, flagged = ? WHERE account_id = ? AND folder_name = ? AND uid = ?",
    )
    .bind(seen as i64)
    .bind(flagged as i64)
    .bind(account_id)
    .bind(folder_name)
    .bind(uid as i64)
    .execute(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to update cache: {}", e))?;

    let total_elapsed = start_time.elapsed();
    println!(
        "✅ Synced flags for UID {} (seen={}, flagged={}) in {:.3}s",
        uid,
        seen,
        flagged,
        total_elapsed.as_secs_f64()
    );

    if result.rows_affected() == 0 {
        println!("⚠️ UID {} not found in local cache", uid);
    }

    Ok(())
}
