use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType, EmailHeader};
use mail_parser::parsers::MessageStream;
use native_tls::TlsConnector;
use tauri::command;

// Helper function to decode RFC 2047 encoded words (e.g., "=?UTF-8?Q?...?=")
fn decode_header(encoded: &str) -> String {
    // Check if the string contains RFC 2047 encoded words
    if !encoded.contains("=?") {
        return encoded.to_string();
    }

    // Use mail-parser's MessageStream to decode RFC 2047 encoded words
    // RFC 2047 format: =?charset?encoding?encoded-text?=
    // The decode_rfc2047 expects: ?charset?encoding?encoded-text?=
    let mut result = String::new();
    let mut remaining = encoded;

    while let Some(start_pos) = remaining.find("=?") {
        // Add any text before the encoded word
        result.push_str(&remaining[..start_pos]);

        // Find the end of the encoded word
        if let Some(end_pos) = remaining[start_pos + 2..].find("?=") {
            // Extract the encoded part: ?charset?encoding?encoded-text?=
            let encoded_part = &remaining[start_pos + 1..start_pos + 2 + end_pos + 2];

            // Decode using mail-parser's MessageStream
            match MessageStream::new(encoded_part.as_bytes()).decode_rfc2047() {
                Some(decoded) => {
                    result.push_str(&decoded);
                }
                None => {
                    // If decoding fails, keep the original
                    result.push_str(&remaining[start_pos..start_pos + 2 + end_pos + 2]);
                }
            }

            remaining = &remaining[start_pos + 2 + end_pos + 2..];
        } else {
            // No proper end found, keep the rest as-is
            result.push_str(&remaining[start_pos..]);
            break;
        }
    }

    // Add any remaining text
    result.push_str(remaining);
    result
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
                    let raw_subject = String::from_utf8_lossy(s).to_string();
                    // Decode RFC 2047 encoded words in subject
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
                                let name = String::from_utf8_lossy(name_bytes).to_string();
                                // Only use the name if it's not empty
                                if !name.trim().is_empty() {
                                    // Decode RFC 2047 encoded words (handles UTF-8, GB2312, etc.)
                                    return decode_header(&name);
                                }
                            }
                            // Fall back to email address if no display name
                            let mailbox = String::from_utf8_lossy(addr.mailbox.unwrap_or_default());
                            let host = String::from_utf8_lossy(addr.host.unwrap_or_default());
                            format!("{}@{}", mailbox, host)
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "(Unknown Sender)".to_string());

            let date = envelope
                .date
                .as_ref()
                .map(|d| String::from_utf8_lossy(d).to_string())
                .unwrap_or_else(|| "(No Date)".to_string());

            let to = envelope
                .to
                .as_ref()
                .map(|addrs| {
                    addrs
                        .iter()
                        .map(|addr| {
                            // Try to get the display name first
                            if let Some(name_bytes) = addr.name {
                                let name = String::from_utf8_lossy(name_bytes).to_string();
                                // Only use the name if it's not empty
                                if !name.trim().is_empty() {
                                    // Decode RFC 2047 encoded words
                                    return decode_header(&name);
                                }
                            }
                            // Fall back to email address if no display name
                            let mailbox = String::from_utf8_lossy(addr.mailbox.unwrap_or_default());
                            let host = String::from_utf8_lossy(addr.host.unwrap_or_default());
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
            });
        }

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
