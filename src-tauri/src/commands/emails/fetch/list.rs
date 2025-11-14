//! Email list fetching operations from IMAP server
//! This module handles retrieving email headers from the IMAP server

use crate::commands::emails::codec::{
    check_for_attachments, decode_bytes_to_string, decode_header, parse_email_date_with_fallback,
};
use crate::commands::emails::imap_helpers;
use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, EmailHeader};
use tauri::command;

/// Fetch email headers from IMAP server (all messages in folder)
#[command]
pub async fn fetch_emails(
    config: AccountConfig,
    folder: Option<String>,
) -> Result<Vec<EmailHeader>, String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    println!("Fetching emails from {} for {}", folder_name, config.email);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;
    let email_for_log = config.email.clone();
    let folder_for_log = folder_name.clone();

    let emails = tokio::task::spawn_blocking(move || -> Result<Vec<EmailHeader>, String> {
        // Use helper function for connection with imap 3.0.0 API
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

        println!("IMAP authentication successful");

        let mailbox = imap_session.select(&folder_name).map_err(|e| {
            eprintln!("❌ Failed to SELECT folder '{}': {}", folder_name, e);
            eprintln!("   This folder may be inaccessible or require special permissions.");
            format!("Cannot access folder '{}': {}", folder_name, e)
        })?;
        println!("{} selected with {} messages", folder_name, mailbox.exists);

        let total = mailbox.exists;
        if total == 0 {
            return Ok(Vec::new());
        }

        // IMAP sequence numbers start at 1, not 0
        // Fetch all messages in the mailbox
        let seq_range = format!("1:{}", total);

        println!(
            "Fetching all {} messages with sequence range: {}",
            total, seq_range
        );

        let messages = imap_session
            .fetch(seq_range, "(UID ENVELOPE BODYSTRUCTURE FLAGS INTERNALDATE)")
            .map_err(|e| e.to_string())?;

        let mut headers = Vec::new();
        for msg in messages.iter().rev() {
            let envelope = msg.envelope().ok_or("No envelope found")?;
            let subject = envelope
                .subject
                .as_ref()
                .map(|s| {
                    // First decode bytes to proper UTF-8 string (handles emoji correctly)
                    let raw_subject = decode_bytes_to_string(s);
                    // Then decode RFC 2047 encoded words if present
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
                            // Try to get the display name first
                            if let Some(ref name_bytes) = addr.name {
                                let name = decode_bytes_to_string(name_bytes.as_ref());
                                // Only use the name if it's not empty
                                if !name.trim().is_empty() {
                                    // Decode RFC 2047 encoded words
                                    return decode_header(&name);
                                }
                            }
                            // Fall back to email address if no display name
                            let mailbox = decode_bytes_to_string(
                                addr.mailbox.clone().unwrap_or_default().as_ref(),
                            );
                            let host = decode_bytes_to_string(
                                addr.host.clone().unwrap_or_default().as_ref(),
                            );
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

            // Parse date to timestamp for sorting and local time conversion
            // Use INTERNALDATE as fallback if Date header parsing fails
            let timestamp = parse_email_date_with_fallback(&date, internal_date.as_deref());

            let to = envelope
                .to
                .as_ref()
                .map(|addrs| {
                    addrs
                        .iter()
                        .map(|addr| {
                            // Try to get the display name first
                            if let Some(ref name_bytes) = addr.name {
                                let name = decode_bytes_to_string(name_bytes.as_ref());
                                // Only use the name if it's not empty
                                if !name.trim().is_empty() {
                                    // Decode RFC 2047 encoded words
                                    return decode_header(&name);
                                }
                            }
                            // Fall back to email address if no display name
                            let mailbox = decode_bytes_to_string(
                                addr.mailbox.clone().unwrap_or_default().as_ref(),
                            );
                            let host = decode_bytes_to_string(
                                addr.host.clone().unwrap_or_default().as_ref(),
                            );
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
                            let host = decode_bytes_to_string(
                                addr.host.clone().unwrap_or_default().as_ref(),
                            );
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

            // Check if email has been flagged/starred by examining FLAGS
            let flagged = msg
                .flags()
                .iter()
                .any(|flag| matches!(flag, imap::types::Flag::Flagged));

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
                flagged,
            });
        }

        // Sort emails by timestamp in descending order (newest first)
        headers.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let _ = imap_session.logout();
        Ok(headers)
    })
    .await
    .map_err(|e| e.to_string())??;

    println!(
        "✅ Fetched {} email headers from {} for {}",
        emails.len(),
        folder_for_log,
        email_for_log
    );
    Ok(emails)
}
