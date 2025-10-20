use crate::models::{AccountConfig, EmailHeader};
use native_tls::TlsConnector;
use tauri::command;

#[command]
pub async fn fetch_emails(config: AccountConfig) -> Result<Vec<EmailHeader>, String> {
    println!("Fetching emails for {}", config.email);
    let email_for_log = config.email.clone();

    let emails = tokio::task::spawn_blocking(move || -> Result<Vec<EmailHeader>, String> {
        let domain = config.imap_server.as_str();
        let port = config.imap_port;
        let email = config.email.as_str();
        let password = config.password.as_str();

        let tls = TlsConnector::builder().build().map_err(|e| e.to_string())?;
        let client = imap::connect((domain, port), domain, &tls).map_err(|e| e.to_string())?;

        let mut imap_session = client.login(email, password).map_err(|e| e.0.to_string())?;
        println!("IMAP login successful");

        let mailbox = imap_session.select("INBOX").map_err(|e| e.to_string())?;
        println!("INBOX selected with {} messages", mailbox.exists);

        let total = mailbox.exists;
        if total == 0 {
            return Ok(Vec::new());
        }

        let start = total.saturating_sub(19);
        let seq_range = format!("{}:{}", start, total);

        let messages = imap_session
            .fetch(seq_range, "(UID ENVELOPE)")
            .map_err(|e| e.to_string())?;

        let mut headers = Vec::new();
        for msg in messages.iter().rev() {
            let envelope = msg.envelope().ok_or("No envelope found")?;
            let subject = envelope
                .subject
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string())
                .unwrap_or_else(|| "(No Subject)".to_string());

            let from = envelope
                .from
                .as_ref()
                .map(|addrs| {
                    addrs
                        .iter()
                        .map(|addr| {
                            format!(
                                "{}",
                                String::from_utf8_lossy(addr.mailbox.unwrap_or_default())
                            )
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

            headers.push(EmailHeader {
                uid: msg.uid.unwrap_or(0),
                subject,
                from,
                date,
            });
        }

        let _ = imap_session.logout();
        Ok(headers)
    })
    .await
    .map_err(|e| e.to_string())??;

    println!(
        "✅ Fetched {} email headers for {}",
        emails.len(),
        email_for_log
    );
    Ok(emails)
}

#[command]
pub async fn fetch_email_body(config: AccountConfig, uid: u32) -> Result<String, String> {
    println!("Fetching body for UID {}", uid);

    let body = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let domain = config.imap_server.as_str();
        let port = config.imap_port;
        let email = config.email.as_str();
        let password = config.password.as_str();

        let tls = TlsConnector::builder().build().map_err(|e| e.to_string())?;
        let client = imap::connect((domain, port), domain, &tls).map_err(|e| e.to_string())?;

        let mut imap_session = client.login(email, password).map_err(|e| e.0.to_string())?;
        imap_session.select("INBOX").map_err(|e| e.to_string())?;

        let messages = imap_session
            .uid_fetch(uid.to_string(), "BODY[]")
            .map_err(|e| e.to_string())?;
        let message = messages.first().ok_or("No message found for UID")?;

        let raw_body = message.body().unwrap_or_default();
        let parsed_mail = mailparse::parse_mail(raw_body).map_err(|e| e.to_string())?;

        let mut html_body = None;
        let mut text_body = None;

        if parsed_mail.ctype.mimetype == "text/html" {
            html_body = Some(parsed_mail.get_body().unwrap_or_default());
        } else if parsed_mail.ctype.mimetype == "text/plain" {
            text_body = Some(parsed_mail.get_body().unwrap_or_default());
        }

        for part in &parsed_mail.subparts {
            if part.ctype.mimetype == "text/html" {
                html_body = Some(part.get_body().unwrap_or_default());
                break;
            } else if part.ctype.mimetype == "text/plain" {
                text_body = Some(part.get_body().unwrap_or_default());
            }
        }

        let final_body = if let Some(body) = html_body {
            body
        } else if let Some(body) = text_body {
            format!("<pre>{}</pre>", html_escape::encode_text(&body))
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
