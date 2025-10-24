// Email fetching operations from IMAP server
// This module handles retrieving email headers and bodies

use crate::commands::emails::cache::{
    load_email_body_from_cache, save_attachments_to_cache, save_email_body_to_cache,
};
use crate::commands::emails::codec::{
    check_for_attachments, decode_bytes_to_string, decode_header, parse_email_date,
};
use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::{AccountConfig, Attachment, AuthType, EmailHeader};
use mail_parser::MimeHeaders;
use native_tls::TlsConnector;
use tauri::command;

/// OAuth2 authenticator for IMAP
pub struct OAuth2 {
    pub user: String,
    pub access_token: String,
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

/// Fetch email headers from IMAP server (last 20 messages)
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
            .fetch(seq_range, "(UID ENVELOPE BODYSTRUCTURE)")
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

            // Check if email has attachments by examining BODYSTRUCTURE
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

/// Internal function that returns both body and attachments
pub async fn fetch_email_body_with_attachments(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<(String, Vec<Attachment>), String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    println!("Fetching body for UID {} from {}", uid, folder_name);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    let (body, attachments) =
        tokio::task::spawn_blocking(move || -> Result<(String, Vec<Attachment>), String> {
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

            // mail-parser automatically handles multipart messages
            let final_body = if let Some(html_body) = parsed_mail.body_html(0) {
                html_body.to_string()
            } else if let Some(text_body) = parsed_mail.body_text(0) {
                format!("<pre>{}</pre>", html_escape::encode_text(&text_body))
            } else {
                "(No readable body found)".to_string()
            };

            // Extract attachments from the email
            let mut attachments = Vec::new();
            for attachment in parsed_mail.attachments() {
                let filename = attachment
                    .attachment_name()
                    .unwrap_or("unnamed_attachment")
                    .to_string();

                let content_type = attachment
                    .content_type()
                    .map(|ct| ct.c_type.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                let data = attachment.contents().to_vec();
                let size = data.len() as i64;

                attachments.push(Attachment {
                    id: None,
                    filename,
                    content_type,
                    size,
                    data: Some(data),
                });
            }

            let _ = imap_session.logout();
            Ok((final_body, attachments))
        })
        .await
        .map_err(|e| e.to_string())??;

    println!(
        "✅ Fetched and parsed body for UID {} with {} attachments",
        uid,
        attachments.len()
    );
    Ok((body, attachments))
}

/// Public command that only returns the body (for backward compatibility)
#[command]
pub async fn fetch_email_body(
    config: AccountConfig,
    uid: u32,
    folder: Option<String>,
) -> Result<String, String> {
    let (body, _) = fetch_email_body_with_attachments(config, uid, folder).await?;
    Ok(body)
}

/// Fetch email body with caching
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
    let (body, attachments) = fetch_email_body_with_attachments(config, uid, folder).await?;

    // Save body to cache
    save_email_body_to_cache(account_id, &folder_name, uid, &body).await?;

    // Save attachments to cache if any
    if !attachments.is_empty() {
        // Get email database ID
        let pool = db::pool();
        let email_id_result = sqlx::query_as::<_, (i64,)>(
            "SELECT id FROM emails WHERE account_id = ? AND folder_name = ? AND uid = ?",
        )
        .bind(account_id)
        .bind(&folder_name)
        .bind(uid as i64)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to get email id: {}", e))?;

        if let Some((email_id,)) = email_id_result {
            save_attachments_to_cache(email_id, &attachments).await?;

            // Update has_attachments flag
            sqlx::query("UPDATE emails SET has_attachments = 1 WHERE id = ?")
                .bind(email_id)
                .execute(pool.as_ref())
                .await
                .map_err(|e| format!("Failed to update has_attachments flag: {}", e))?;
        }
    }

    Ok(body)
}
