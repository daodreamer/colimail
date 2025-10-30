// IMAP batch fetching logic with retry and reconnection mechanisms
// This module handles fetching emails in batches with adaptive batch sizing

use super::parse::parse_email_headers;
use crate::commands::emails::imap_helpers;
use crate::models::{AccountConfig, EmailHeader};

/// Check if an IMAP error is a connection error (Bye) that requires reconnection
pub fn is_connection_error(error: &imap::Error) -> bool {
    matches!(error, imap::Error::Bye(_))
}

/// Fetch all emails from folder using sequence numbers (full sync)
/// Uses adaptive batch sizing with reconnection on Bye errors
pub fn fetch_all_by_sequence(
    imap_session: &mut imap::Session<Box<dyn imap::ImapConnection>>,
    server_exists: u32,
    config: &AccountConfig,
    folder_name: &str,
) -> Result<Vec<EmailHeader>, String> {
    if server_exists == 0 {
        return Ok(Vec::new());
    }

    // Fetch in batches to avoid overwhelming the IMAP server and parser
    // Start with batch size 20, increase exponentially until hitting server limit
    let mut batch_size = 20u32;
    let mut max_batch_size: Option<u32> = None; // Lock batch size after first Bye error
    let mut all_headers = Vec::new();
    let mut current_pos = 1u32;

    println!(
        "📥 Fetching all {} messages (starting batch size: {})",
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
            batch_num, seq_range, count
        );

        // Fetch without BODYSTRUCTURE (causes issues with GMX)
        match imap_session.fetch(
            seq_range.as_str(),
            "(UID ENVELOPE FLAGS INTERNALDATE RFC822.SIZE)",
        ) {
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
                        println!(
                            "  📈 Increasing batch size to {} (locked max: {})",
                            batch_size, max
                        );
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
                        println!(
                            "  🔒 Locking batch size to last successful: {}",
                            last_successful_size
                        );
                    }

                    // Try to logout the old session gracefully (ignore errors)
                    let _ = imap_session.logout();

                    // Wait 2 seconds before reconnecting
                    println!("  ⏱️ Waiting 2 seconds before reconnecting...");
                    std::thread::sleep(std::time::Duration::from_secs(2));

                    // Reconnect to IMAP server
                    match imap_helpers::connect_and_login(config) {
                        Ok(new_session) => {
                            *imap_session = new_session;
                            println!("  ✅ Reconnected successfully");

                            // Re-select the folder
                            match imap_session.select(folder_name) {
                                Ok(_) => {
                                    println!("  ✅ Folder re-selected");
                                    // Use the locked batch size
                                    batch_size = last_successful_size;
                                    println!("  ⚠️ Retrying with safe batch size: {}", batch_size);
                                    // Don't increment current_pos, we'll retry this range
                                }
                                Err(select_err) => {
                                    return Err(format!(
                                        "Failed to re-select folder after reconnection: {}",
                                        select_err
                                    ));
                                }
                            }
                        }
                        Err(conn_err) => {
                            return Err(format!(
                                "Failed to reconnect after Bye error: {}",
                                conn_err
                            ));
                        }
                    }
                } else {
                    // Not a connection error, just reduce batch size

                    // If batch size is already very small, give up
                    if batch_size <= 10 {
                        return Err(format!(
                            "Failed to fetch batch {} even with minimum batch size: {}",
                            batch_num, e
                        ));
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

    println!(
        "✅ Fetched {} emails in {} batch(es)",
        all_headers.len(),
        batch_num
    );
    Ok(all_headers)
}

/// Fetch new emails by UID list (incremental sync)
/// Uses adaptive batch sizing with retry on errors
pub fn fetch_new_by_uid_list(
    imap_session: &mut imap::Session<Box<dyn imap::ImapConnection>>,
    new_uids: Vec<u32>,
    highest_uid: i64,
) -> Result<Vec<EmailHeader>, String> {
    if new_uids.is_empty() {
        return Ok(Vec::new());
    }

    println!(
        "📥 Fetching {} new message(s) with UIDs: {:?}",
        new_uids.len(),
        new_uids
    );

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

        println!(
            "  📦 Batch {}: fetching {} message(s)",
            batch_num, chunk_size
        );

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
                    println!(
                        "  ✨ Batch {} found {} raw message(s) from IMAP",
                        batch_num, count
                    );

                    // Debug: Log the actual UIDs returned by IMAP
                    for (idx, msg) in messages.iter().enumerate() {
                        println!("    📋 Message {}: UID = {:?}", idx + 1, msg.uid);
                    }

                    let parsed = parse_email_headers(messages.iter().rev());

                    // Debug: Log the parsed UIDs
                    println!(
                        "    📝 Parsed UIDs: {:?}",
                        parsed.iter().map(|e| e.uid).collect::<Vec<_>>()
                    );

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

                println!(
                    "  ✓ Batch {} complete, {} total new emails so far",
                    batch_num,
                    all_new_headers.len()
                );

                // Move to next batch
                current_idx = end_idx;

                // Gradually increase batch size if successful
                // If max_batch_size is set (after a Bye error), respect that limit
                if let Some(max) = max_batch_size {
                    if batch_size < max {
                        batch_size = (batch_size * 2).min(max);
                        println!(
                            "  📈 Increasing batch size to {} (locked max: {})",
                            batch_size, max
                        );
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
                        println!(
                            "  🔒 Locking batch size to last successful: {}",
                            last_successful_size
                        );
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
                        eprintln!(
                            "  ❌ Skipping batch {} (already at minimum batch size)",
                            batch_num
                        );
                        current_idx = end_idx;
                    }
                }
            }
        }
    }

    if all_new_headers.is_empty() {
        println!("✅ No new messages after fetching all batches");
    } else {
        println!(
            "✨ Total {} genuinely new message(s) from all batches",
            all_new_headers.len()
        );
    }

    Ok(all_new_headers)
}
