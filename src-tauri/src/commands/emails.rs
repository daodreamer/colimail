use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType, EmailHeader};
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
