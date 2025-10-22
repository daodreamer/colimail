use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::{AccountConfig, AuthType, EmailHeader};
use chrono::{DateTime, Utc};
use encoding_rs::Encoding;
use native_tls::TlsConnector;
use tauri::command;

// Helper function to decode RFC 2047 encoded words (e.g., "=?UTF-8?Q?...?=")
// RFC 2047 format: =?charset?encoding?encoded-text?=
// where encoding can be Q (Quoted-Printable) or B (Base64)
fn decode_header(encoded: &str) -> String {
    // Check if the string contains RFC 2047 encoded words
    if !encoded.contains("=?") {
        return encoded.to_string();
    }

    let mut result = String::new();
    let mut remaining = encoded;

    while let Some(start_pos) = remaining.find("=?") {
        // Add any text before the encoded word
        result.push_str(&remaining[..start_pos]);

        // RFC 2047 format: =?charset?encoding?encoded-text?=
        // Parse step by step to avoid finding ? or = within the encoded content
        let after_start = &remaining[start_pos + 2..];

        // Find the first ? (end of charset)
        if let Some(charset_end) = after_start.find('?') {
            let charset = &after_start[..charset_end];
            let after_charset = &after_start[charset_end + 1..];

            // Find the second ? (end of encoding)
            if let Some(encoding_end) = after_charset.find('?') {
                let encoding = &after_charset[..encoding_end];
                let after_encoding = &after_charset[encoding_end + 1..];

                // Find ?= (end of encoded text)
                if let Some(text_end) = after_encoding.find("?=") {
                    let encoded_text = &after_encoding[..text_end];

                    // Calculate the full length of the encoded word
                    let full_length =
                        2 + charset.len() + 1 + encoding.len() + 1 + encoded_text.len() + 2;
                    let full_encoded = &remaining[start_pos..start_pos + full_length];

                    let encoding_upper = encoding.to_uppercase();

                    let decoded = match encoding_upper.as_str() {
                        "Q" => decode_quoted_printable(encoded_text),
                        "B" => decode_base64(encoded_text),
                        _ => None,
                    };

                    if let Some(decoded_bytes) = decoded {
                        // Convert bytes to string using the specified charset
                        // Use encoding_rs to handle various character encodings
                        let encoding = Encoding::for_label(charset.as_bytes());

                        let decoded_str = if let Some(enc) = encoding {
                            // Use encoding_rs to decode
                            let (cow, _encoding_used, _had_errors) = enc.decode(&decoded_bytes);
                            Some(cow.into_owned())
                        } else {
                            // If encoding not recognized, try UTF-8 as fallback
                            String::from_utf8(decoded_bytes).ok()
                        };

                        if let Some(s) = decoded_str {
                            result.push_str(&s);
                        } else {
                            // Decoding failed, keep original
                            result.push_str(full_encoded);
                        }
                    } else {
                        // Decoding failed, keep original
                        result.push_str(full_encoded);
                    }

                    // Move past the encoded word
                    remaining = &remaining[start_pos + full_length..];

                    // RFC 2047: whitespace between encoded words should be ignored
                    if remaining.starts_with(' ')
                        && remaining.len() > 1
                        && remaining[1..].starts_with("=?")
                    {
                        remaining = &remaining[1..];
                    }
                } else {
                    // No ?= found, not a valid encoded word
                    result.push_str(&remaining[start_pos..start_pos + 2]);
                    remaining = &remaining[start_pos + 2..];
                }
            } else {
                // No second ? found
                result.push_str(&remaining[start_pos..start_pos + 2]);
                remaining = &remaining[start_pos + 2..];
            }
        } else {
            // No first ? found
            result.push_str(&remaining[start_pos..start_pos + 2]);
            remaining = &remaining[start_pos + 2..];
        }
    }

    // Add any remaining text
    result.push_str(remaining);
    result
}

// Decode Quoted-Printable (Q encoding) for RFC 2047
fn decode_quoted_printable(encoded: &str) -> Option<Vec<u8>> {
    let mut decoded = Vec::new();
    let mut chars = encoded.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '_' => decoded.push(b' '), // underscore represents space in Q encoding
            '=' => {
                // Get next two hex digits
                if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                    if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                        decoded.push(byte);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            _ if ch.is_ascii() => decoded.push(ch as u8),
            _ => return None, // Non-ASCII in Q encoding is invalid
        }
    }

    Some(decoded)
}

// Decode Base64 (B encoding) for RFC 2047
fn decode_base64(encoded: &str) -> Option<Vec<u8>> {
    // Simple base64 decoding
    use std::collections::HashMap;

    let b64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut char_map = HashMap::new();
    for (i, c) in b64_chars.chars().enumerate() {
        char_map.insert(c, i as u8);
    }

    let mut decoded = Vec::new();
    let chars: Vec<char> = encoded.chars().filter(|c| !c.is_whitespace()).collect();

    for chunk in chars.chunks(4) {
        if chunk.len() < 2 {
            return None;
        }

        let b1 = char_map.get(&chunk[0])?;
        let b2 = char_map.get(&chunk[1])?;

        decoded.push((b1 << 2) | (b2 >> 4));

        if chunk.len() > 2 && chunk[2] != '=' {
            let b3 = char_map.get(&chunk[2])?;
            decoded.push(((b2 & 0x0F) << 4) | (b3 >> 2));

            if chunk.len() > 3 && chunk[3] != '=' {
                let b4 = char_map.get(&chunk[3])?;
                decoded.push(((b3 & 0x03) << 6) | b4);
            }
        }
    }

    Some(decoded)
}

// Helper function to safely decode bytes to UTF-8 string
// Handles both raw UTF-8 and potential encoding issues with emoji
fn decode_bytes_to_string(bytes: &[u8]) -> String {
    // First try to parse as valid UTF-8
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            // If invalid UTF-8, use lossy conversion but be more careful
            String::from_utf8_lossy(bytes).to_string()
        }
    }
}

// Parse RFC 2822 date string to Unix timestamp
// Email dates are in format like: "Mon, 15 Jan 2024 14:30:00 +0800"
fn parse_email_date(date_str: &str) -> i64 {
    // Try to parse the RFC 2822 format date
    if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
        return dt.timestamp();
    }

    // Try alternative RFC 3339 format (ISO 8601)
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return dt.timestamp();
    }

    // If parsing fails, try to extract timestamp from various formats
    // Some servers might send non-standard date formats
    if let Ok(dt) = chrono::DateTime::parse_from_str(date_str, "%a, %d %b %Y %H:%M:%S %z") {
        return dt.timestamp();
    }

    // If all parsing fails, return current timestamp as fallback
    eprintln!("⚠️ Failed to parse date: {}", date_str);
    Utc::now().timestamp()
}

// OAuth2 authenticator for IMAP
struct OAuth2 {
    user: String,
    access_token: String,
}

impl imap::Authenticator for OAuth2 {
    type Response = String;
    fn process(&self, _: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}

// Helper function to find the trash/deleted folder for an account
// Different providers use different names for the trash folder
fn find_trash_folder(
    imap_session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
) -> Result<String, String> {
    // Common trash folder names used by different providers
    let trash_candidates = vec![
        "[Gmail]/Trash",           // Gmail
        "[Gmail]/Bin",             // Gmail (some locales)
        "Trash",                   // Standard IMAP
        "Deleted",                 // Common alternative
        "Deleted Items",           // Outlook/Exchange
        "Deleted Messages",        // Some providers
        "[Outlook]/Deleted Items", // Outlook with prefix
        "INBOX.Trash",             // Some IMAP servers
        "INBOX.Deleted",           // Some IMAP servers
        "垃圾箱",                  // Chinese
        "已删除邮件",              // Chinese (Outlook style)
    ];

    println!("Searching for trash folder...");

    // List all folders
    let mailbox_list = imap_session
        .list(Some(""), Some("*"))
        .map_err(|e| format!("Failed to list folders: {}", e))?;

    // Try to find a trash folder by matching against known names
    for mailbox in mailbox_list.iter() {
        let folder_name = mailbox.name();
        let lower_name = folder_name.to_lowercase();

        // Check if this folder matches any of our trash candidates
        for candidate in &trash_candidates {
            if folder_name == *candidate
                || lower_name.contains("trash")
                || lower_name.contains("deleted")
                || lower_name.contains("垃圾")
            {
                // Verify the folder is selectable
                if !mailbox
                    .attributes()
                    .iter()
                    .any(|attr| matches!(attr, imap::types::NameAttribute::NoSelect))
                {
                    println!("Found trash folder: {}", folder_name);
                    return Ok(folder_name.to_string());
                }
            }
        }
    }

    // If no trash folder found, return an error
    Err("Could not find trash/deleted folder. The email provider may not have a standard trash folder.".to_string())
}

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

                println!("🔐 Attempting OAuth2 authentication for: {}", email);
                println!("   Server: {}:{}", domain, port);
                println!("   Token length: {} chars", access_token.len());

                // OAuth2 XOAUTH2 authentication
                let oauth2 = OAuth2 {
                    user: email.to_string(),
                    access_token: access_token.clone(),
                };

                client.authenticate("XOAUTH2", &oauth2).map_err(|e| {
                    eprintln!("❌ OAuth2 authentication failed: {}", e.0);
                    eprintln!("   Possible causes:");
                    eprintln!("   1. Expired or invalid access token");
                    eprintln!("   2. Incorrect OAuth2 scopes in Azure AD");
                    eprintln!("   3. IMAP not enabled for this mailbox");
                    format!("OAuth2 authentication failed: {}", e.0)
                })?
            }
            _ => {
                let password = config
                    .password
                    .as_ref()
                    .ok_or("Password is required for basic authentication")?;

                client.login(email, password).map_err(|e| e.0.to_string())?
            }
        };

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
        // Get last 20 messages (or all if less than 20)
        let start = total.saturating_sub(19).max(1);
        let seq_range = format!("{}:{}", start, total);

        println!("Fetching messages with sequence range: {}", seq_range);

        let messages = imap_session
            .fetch(seq_range, "(UID ENVELOPE)")
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
                            // Try to get the display name first (e.g., "Lisa Stein von Stepstone")
                            if let Some(name_bytes) = addr.name {
                                let name = decode_bytes_to_string(name_bytes);
                                // Only use the name if it's not empty
                                if !name.trim().is_empty() {
                                    // Decode RFC 2047 encoded words (handles UTF-8, GB2312, etc.)
                                    return decode_header(&name);
                                }
                            }
                            // Fall back to email address if no display name
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

            // Parse date to timestamp for sorting and local time conversion
            let timestamp = parse_email_date(&date);

            let to = envelope
                .to
                .as_ref()
                .map(|addrs| {
                    addrs
                        .iter()
                        .map(|addr| {
                            // Try to get the display name first
                            if let Some(name_bytes) = addr.name {
                                let name = decode_bytes_to_string(name_bytes);
                                // Only use the name if it's not empty
                                if !name.trim().is_empty() {
                                    // Decode RFC 2047 encoded words
                                    return decode_header(&name);
                                }
                            }
                            // Fall back to email address if no display name
                            let mailbox = decode_bytes_to_string(addr.mailbox.unwrap_or_default());
                            let host = decode_bytes_to_string(addr.host.unwrap_or_default());
                            format!("{}@{}", mailbox, host)
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "(Unknown Recipient)".to_string());

            headers.push(EmailHeader {
                uid: msg.uid.unwrap_or(0),
                subject,
                from,
                to,
                date,
                timestamp,
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

#[command]
pub async fn fetch_email_body(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<String, String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    println!("Fetching body for UID {} from {}", uid, folder_name);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    let body = tokio::task::spawn_blocking(move || -> Result<String, String> {
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

                println!("🔐 Attempting OAuth2 authentication for: {}", email);
                println!("   Server: {}:{}", domain, port);
                println!("   Token length: {} chars", access_token.len());

                // OAuth2 XOAUTH2 authentication
                let oauth2 = OAuth2 {
                    user: email.to_string(),
                    access_token: access_token.clone(),
                };

                client.authenticate("XOAUTH2", &oauth2).map_err(|e| {
                    eprintln!("❌ OAuth2 authentication failed: {}", e.0);
                    eprintln!("   Possible causes:");
                    eprintln!("   1. Expired or invalid access token");
                    eprintln!("   2. Incorrect OAuth2 scopes in Azure AD");
                    eprintln!("   3. IMAP not enabled for this mailbox");
                    format!("OAuth2 authentication failed: {}", e.0)
                })?
            }
            _ => {
                let password = config
                    .password
                    .as_ref()
                    .ok_or("Password is required for basic authentication")?;

                client.login(email, password).map_err(|e| e.0.to_string())?
            }
        };

        imap_session.select(&folder_name).map_err(|e| {
            eprintln!(
                "❌ Failed to SELECT folder '{}' for UID {}: {}",
                folder_name, uid, e
            );
            format!("Cannot access folder '{}': {}", folder_name, e)
        })?;

        let messages = imap_session
            .uid_fetch(uid.to_string(), "BODY[]")
            .map_err(|e| e.to_string())?;
        let message = messages.first().ok_or("No message found for UID")?;

        let raw_body = message.body().unwrap_or_default();
        let parsed_mail = mail_parser::MessageParser::default()
            .parse(raw_body)
            .ok_or("Failed to parse email message")?;

        // mail-parser automatically handles multipart messages and provides
        // body_html() and body_text() methods that extract the appropriate content
        let final_body = if let Some(html_body) = parsed_mail.body_html(0) {
            html_body.to_string()
        } else if let Some(text_body) = parsed_mail.body_text(0) {
            format!("<pre>{}</pre>", html_escape::encode_text(&text_body))
        } else {
            "(No readable body found)".to_string()
        };

        let _ = imap_session.logout();
        Ok(final_body)
    })
    .await
    .map_err(|e| e.to_string())??;

    println!("✅ Fetched and parsed body for UID {}", uid);
    Ok(body)
}

#[command]
pub async fn move_email_to_trash(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<(), String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    println!("Moving email UID {} from {} to trash", uid, folder_name);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    tokio::task::spawn_blocking(move || -> Result<(), String> {
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

                println!("🔐 Attempting OAuth2 authentication for: {}", email);

                let oauth2 = OAuth2 {
                    user: email.to_string(),
                    access_token: access_token.clone(),
                };

                client.authenticate("XOAUTH2", &oauth2).map_err(|e| {
                    eprintln!("❌ OAuth2 authentication failed: {}", e.0);
                    format!("OAuth2 authentication failed: {}", e.0)
                })?
            }
            _ => {
                let password = config
                    .password
                    .as_ref()
                    .ok_or("Password is required for basic authentication")?;

                client.login(email, password).map_err(|e| e.0.to_string())?
            }
        };

        println!("IMAP authentication successful");

        // Find the trash folder
        let trash_folder = find_trash_folder(&mut imap_session)?;
        println!("Using trash folder: {}", trash_folder);

        // Select the source folder
        imap_session.select(&folder_name).map_err(|e| {
            eprintln!("❌ Failed to SELECT folder '{}': {}", folder_name, e);
            format!("Cannot access folder '{}': {}", folder_name, e)
        })?;

        // Copy the email to trash folder using UID COPY
        imap_session
            .uid_copy(format!("{}", uid), &trash_folder)
            .map_err(|e| {
                eprintln!("❌ Failed to copy UID {} to trash: {}", uid, e);
                format!("Failed to copy email to trash: {}", e)
            })?;

        println!("Copied UID {} to trash folder", uid);

        // Mark the original email for deletion using UID STORE
        imap_session
            .uid_store(format!("{}", uid), "+FLAGS (\\Deleted)")
            .map_err(|e| {
                eprintln!("❌ Failed to mark UID {} as deleted: {}", uid, e);
                format!("Failed to mark email as deleted: {}", e)
            })?;

        println!("Marked UID {} with \\Deleted flag", uid);

        // Permanently delete (expunge) the original
        imap_session.expunge().map_err(|e| {
            eprintln!("❌ Failed to expunge deleted messages: {}", e);
            format!("Failed to remove email from original folder: {}", e)
        })?;

        println!("✅ Successfully moved email to trash");

        let _ = imap_session.logout();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    println!("✅ Successfully moved email UID {} to trash", uid);

    Ok(())
}

#[command]
pub async fn delete_email(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<(), String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    println!(
        "Permanently deleting email UID {} from {}",
        uid, folder_name
    );

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    tokio::task::spawn_blocking(move || -> Result<(), String> {
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

                println!("🔐 Attempting OAuth2 authentication for: {}", email);

                let oauth2 = OAuth2 {
                    user: email.to_string(),
                    access_token: access_token.clone(),
                };

                client.authenticate("XOAUTH2", &oauth2).map_err(|e| {
                    eprintln!("❌ OAuth2 authentication failed: {}", e.0);
                    format!("OAuth2 authentication failed: {}", e.0)
                })?
            }
            _ => {
                let password = config
                    .password
                    .as_ref()
                    .ok_or("Password is required for basic authentication")?;

                client.login(email, password).map_err(|e| e.0.to_string())?
            }
        };

        println!("IMAP authentication successful");

        // Select the folder
        imap_session.select(&folder_name).map_err(|e| {
            eprintln!("❌ Failed to SELECT folder '{}': {}", folder_name, e);
            format!("Cannot access folder '{}': {}", folder_name, e)
        })?;

        // Mark the email for deletion using UID
        imap_session
            .uid_store(format!("{}", uid), "+FLAGS (\\Deleted)")
            .map_err(|e| {
                eprintln!("❌ Failed to mark UID {} as deleted: {}", uid, e);
                format!("Failed to mark email as deleted: {}", e)
            })?;

        println!("Marked UID {} with \\Deleted flag", uid);

        // Permanently delete messages marked with \Deleted flag
        imap_session.expunge().map_err(|e| {
            eprintln!("❌ Failed to expunge deleted messages: {}", e);
            format!("Failed to permanently delete email: {}", e)
        })?;

        println!("✅ Successfully expunged deleted messages");

        let _ = imap_session.logout();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    println!("✅ Successfully deleted email UID {} from server", uid);

    Ok(())
}

// Save emails to database cache
async fn save_emails_to_cache(
    account_id: i32,
    folder_name: &str,
    emails: &[EmailHeader],
) -> Result<(), String> {
    let pool = db::pool();
    let current_time = Utc::now().timestamp();

    for email in emails {
        // Use INSERT OR REPLACE to update existing emails
        sqlx::query(
            "INSERT OR REPLACE INTO emails
            (account_id, folder_name, uid, subject, from_addr, to_addr, date, timestamp, synced_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(account_id)
        .bind(folder_name)
        .bind(email.uid as i64)
        .bind(&email.subject)
        .bind(&email.from)
        .bind(&email.to)
        .bind(&email.date)
        .bind(email.timestamp)
        .bind(current_time)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to save email to cache: {}", e))?;
    }

    println!(
        "✅ Saved {} emails to cache for folder {}",
        emails.len(),
        folder_name
    );
    Ok(())
}

// Load emails from database cache
#[command]
pub async fn load_emails_from_cache(
    account_id: i32,
    folder: Option<String>,
) -> Result<Vec<EmailHeader>, String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    println!(
        "Loading emails from cache for account {} folder {}",
        account_id, folder_name
    );

    let pool = db::pool();

    let rows = sqlx::query_as::<_, (i64, String, String, String, String, i64)>(
        "SELECT uid, subject, from_addr, to_addr, date, timestamp
        FROM emails
        WHERE account_id = ? AND folder_name = ?
        ORDER BY timestamp DESC",
    )
    .bind(account_id)
    .bind(&folder_name)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to load emails from cache: {}", e))?;

    let emails: Vec<EmailHeader> = rows
        .into_iter()
        .map(|(uid, subject, from, to, date, timestamp)| EmailHeader {
            uid: uid as u32,
            subject,
            from,
            to,
            date,
            timestamp,
        })
        .collect();

    println!(
        "✅ Loaded {} emails from cache for folder {}",
        emails.len(),
        folder_name
    );
    Ok(emails)
}

// Sync emails from server and update cache
#[command]
pub async fn sync_emails(
    config: AccountConfig,
    folder: Option<String>,
) -> Result<Vec<EmailHeader>, String> {
    let account_id = config.id.ok_or("Account ID is required")?;
    let folder_name = folder.clone().unwrap_or_else(|| "INBOX".to_string());

    println!(
        "Syncing emails from server for account {} folder {}",
        account_id, folder_name
    );

    // Fetch emails from server
    let emails = fetch_emails(config, folder).await?;

    // Save to cache
    save_emails_to_cache(account_id, &folder_name, &emails).await?;

    // Update last sync time
    update_last_sync_time(account_id, &folder_name).await?;

    Ok(emails)
}

// Update last sync time for a folder
async fn update_last_sync_time(account_id: i32, folder_name: &str) -> Result<(), String> {
    let pool = db::pool();
    let current_time = Utc::now().timestamp();

    sqlx::query(
        "INSERT OR REPLACE INTO sync_status (account_id, folder_name, last_sync_time)
        VALUES (?, ?, ?)",
    )
    .bind(account_id)
    .bind(folder_name)
    .bind(current_time)
    .execute(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to update sync time: {}", e))?;

    Ok(())
}

// Get last sync time for a folder
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

// Check if sync is needed based on interval
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

// Save email body to cache
async fn save_email_body_to_cache(
    account_id: i32,
    folder_name: &str,
    uid: u32,
    body: &str,
) -> Result<(), String> {
    let pool = db::pool();

    sqlx::query("UPDATE emails SET body = ? WHERE account_id = ? AND folder_name = ? AND uid = ?")
        .bind(body)
        .bind(account_id)
        .bind(folder_name)
        .bind(uid as i64)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to save body to cache: {}", e))?;

    println!("✅ Saved body to cache for UID {}", uid);
    Ok(())
}

// Load email body from cache
async fn load_email_body_from_cache(
    account_id: i32,
    folder_name: &str,
    uid: u32,
) -> Result<Option<String>, String> {
    let pool = db::pool();

    let result = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT body FROM emails WHERE account_id = ? AND folder_name = ? AND uid = ?",
    )
    .bind(account_id)
    .bind(folder_name)
    .bind(uid as i64)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to load body from cache: {}", e))?;

    Ok(result.and_then(|(body,)| body))
}

// Fetch email body with caching
#[command]
pub async fn fetch_email_body_cached(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<String, String> {
    let account_id = config.id.ok_or("Account ID is required")?;
    let folder_name = folder.clone().unwrap_or_else(|| "INBOX".to_string());

    // Try to load from cache first
    if let Some(cached_body) = load_email_body_from_cache(account_id, &folder_name, uid).await? {
        println!("✅ Loaded body from cache for UID {}", uid);
        return Ok(cached_body);
    }

    println!("📥 Fetching body from server for UID {}", uid);

    // Not in cache, fetch from server
    let body = fetch_email_body(config, uid, folder).await?;

    // Save to cache
    save_email_body_to_cache(account_id, &folder_name, uid, &body).await?;

    Ok(body)
}

// Get sync interval setting
#[command]
pub async fn get_sync_interval() -> Result<i64, String> {
    let pool = db::pool();

    let result =
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'sync_interval'")
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to get sync interval: {}", e))?;

    let interval_str = result.map(|(v,)| v).unwrap_or_else(|| "300".to_string());
    interval_str
        .parse::<i64>()
        .map_err(|e| format!("Failed to parse sync interval: {}", e))
}

// Set sync interval setting
#[command]
pub async fn set_sync_interval(interval: i64) -> Result<(), String> {
    let pool = db::pool();

    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('sync_interval', ?)")
        .bind(interval.to_string())
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to set sync interval: {}", e))?;

    println!("✅ Set sync interval to {} seconds", interval);
    Ok(())
}
