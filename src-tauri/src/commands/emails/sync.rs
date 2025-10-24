// Email synchronization logic
// This module handles incremental sync using UIDVALIDITY and UIDs

use crate::commands::emails::cache::{load_emails_from_cache, save_emails_to_cache};
use crate::commands::emails::codec::{
    check_for_attachments, decode_bytes_to_string, decode_header, parse_email_date,
};
use crate::commands::emails::fetch::OAuth2;
use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::{AccountConfig, AuthType, EmailHeader};
use chrono::Utc;
use native_tls::TlsConnector;
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
    let emails = incremental_sync(config, account_id, &folder_name).await?;

    println!(
        "✅ Incremental sync completed: {} emails in cache",
        emails.len()
    );

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

    // Connect to IMAP and check current state
    let (server_uidvalidity, _server_exists, new_emails) =
        tokio::task::spawn_blocking(move || -> Result<(u32, u32, Vec<EmailHeader>), String> {
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

                    println!("🔐 OAuth2 authentication for incremental sync: {}", email);

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
            let new_emails = if sync_state_for_task.is_none()
                || sync_state_for_task.as_ref().unwrap().uidvalidity
                    != Some(server_uidvalidity as i64)
            {
                // Full sync needed: no previous state or UIDVALIDITY changed
                if sync_state_for_task.is_some() {
                    println!("⚠️ UIDVALIDITY changed! Full resync required.");
                } else {
                    println!("🆕 First sync for this folder.");
                }

                // Fetch recent emails (last 100)
                if server_exists == 0 {
                    Vec::new()
                } else {
                    let start = server_exists.saturating_sub(99).max(1);
                    let seq_range = format!("{}:{}", start, server_exists);

                    println!("📥 Fetching messages: {}", seq_range);

                    let messages = imap_session
                        .fetch(seq_range, "(UID ENVELOPE BODYSTRUCTURE)")
                        .map_err(|e| e.to_string())?;

                    parse_email_headers(messages.iter().rev())
                }
            } else {
                // Incremental sync: fetch only new messages
                let highest_uid = sync_state_for_task
                    .as_ref()
                    .unwrap()
                    .highest_uid
                    .unwrap_or(0);

                println!("🔄 Incremental sync from UID > {}", highest_uid);

                if highest_uid == 0 || server_exists == 0 {
                    // No previous emails or empty folder
                    Vec::new()
                } else {
                    // Fetch new messages: UID > highest_uid
                    let uid_range = format!("{}:*", highest_uid + 1);

                    println!("📥 Fetching new messages: UID {}", uid_range);

                    match imap_session.uid_fetch(uid_range, "(UID ENVELOPE BODYSTRUCTURE)") {
                        Ok(messages) => {
                            let count = messages.len();
                            if count > 0 {
                                println!("✨ Found {} new message(s)", count);
                                parse_email_headers(messages.iter().rev())
                            } else {
                                println!("✅ No new messages");
                                Vec::new()
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️ Failed to fetch new messages: {}", e);
                            Vec::new()
                        }
                    }
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
    let new_highest_uid = new_emails
        .iter()
        .map(|e| e.uid)
        .max()
        .or(sync_state
            .as_ref()
            .and_then(|s| s.highest_uid.map(|u| u as u32)))
        .unwrap_or(0);

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

/// Helper function to parse IMAP fetch results into EmailHeader
fn parse_email_headers<'a, I>(messages: I) -> Vec<EmailHeader>
where
    I: Iterator<Item = &'a imap::types::Fetch>,
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
                let raw_subject = decode_bytes_to_string(s);
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
                        if let Some(name_bytes) = addr.name {
                            let name = decode_bytes_to_string(name_bytes);
                            if !name.trim().is_empty() {
                                return decode_header(&name);
                            }
                        }
                        let mailbox = decode_bytes_to_string(addr.mailbox.unwrap_or_default());
                        let host = decode_bytes_to_string(addr.host.unwrap_or_default());
                        format!("{}@{}", mailbox, host)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "(Unknown Sender)".to_string());

        let date = envelope
            .date
            .as_ref()
            .map(|d| decode_bytes_to_string(d))
            .unwrap_or_else(|| "(No Date)".to_string());

        let timestamp = parse_email_date(&date);

        let to = envelope
            .to
            .as_ref()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        if let Some(name_bytes) = addr.name {
                            let name = decode_bytes_to_string(name_bytes);
                            if !name.trim().is_empty() {
                                return decode_header(&name);
                            }
                        }
                        let mailbox = decode_bytes_to_string(addr.mailbox.unwrap_or_default());
                        let host = decode_bytes_to_string(addr.host.unwrap_or_default());
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
                        if let Some(name_bytes) = addr.name {
                            let name = decode_bytes_to_string(name_bytes);
                            if !name.trim().is_empty() {
                                return decode_header(&name);
                            }
                        }
                        let mailbox = decode_bytes_to_string(addr.mailbox.unwrap_or_default());
                        let host = decode_bytes_to_string(addr.host.unwrap_or_default());
                        format!("{}@{}", mailbox, host)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "".to_string());

        let has_attachments = msg
            .bodystructure()
            .map(|bs| check_for_attachments(bs))
            .unwrap_or(false);

        headers.push(EmailHeader {
            uid: msg.uid.unwrap_or(0),
            subject,
            from,
            to,
            cc,
            date,
            timestamp,
            has_attachments,
        });
    }

    // Sort by timestamp descending (newest first)
    headers.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

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
