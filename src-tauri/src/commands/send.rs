use crate::attachment_limits::{get_limit_for_email, validate_attachment_sizes};
use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType};
use lettre::{
    message::{Attachment as LettreAttachment, Body, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tauri::command;

#[derive(serde::Deserialize)]
pub struct AttachmentData {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

#[command]
pub async fn send_email(
    config: AccountConfig,
    to: String,
    subject: String,
    body: String,
    attachments: Option<Vec<AttachmentData>>,
) -> Result<String, String> {
    println!("Sending email to {}", to);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    let from: Mailbox = config.email.parse::<Mailbox>().map_err(|e| e.to_string())?;
    let to_mailbox: Mailbox = to.parse::<Mailbox>().map_err(|e| e.to_string())?;

    let email_builder = Message::builder()
        .from(from)
        .to(to_mailbox)
        .subject(subject);

    // Validate attachment sizes if attachments are present
    if let Some(ref attachment_list) = attachments {
        if !attachment_list.is_empty() {
            validate_attachment_sizes(&config.email, attachment_list)?;
        }
    }

    // Build multipart message if there are attachments
    let email = if let Some(attachment_list) = attachments {
        if !attachment_list.is_empty() {
            let mut multipart = MultiPart::mixed().singlepart(SinglePart::html(body));

            for attachment_data in attachment_list {
                let attachment_body = Body::new(attachment_data.data);
                let attachment = LettreAttachment::new(attachment_data.filename).body(
                    attachment_body,
                    attachment_data.content_type.parse().unwrap(),
                );
                multipart = multipart.singlepart(attachment);
            }

            email_builder
                .multipart(multipart)
                .map_err(|e| e.to_string())?
        } else {
            // Send as HTML even without attachments
            email_builder
                .multipart(MultiPart::alternative().singlepart(SinglePart::html(body)))
                .map_err(|e| e.to_string())?
        }
    } else {
        // Send as HTML even without attachments
        email_builder
            .multipart(MultiPart::alternative().singlepart(SinglePart::html(body)))
            .map_err(|e| e.to_string())?
    };

    let mailer = match config.auth_type {
        Some(AuthType::OAuth2) => {
            let access_token = config
                .access_token
                .clone()
                .ok_or("Access token is required for OAuth2 authentication")?;

            println!(
                "🔐 Building SMTP transport with XOAUTH2 for {}",
                config.email
            );
            println!("   Server: {}:{}", config.smtp_server, config.smtp_port);
            println!("   Token length: {} chars", access_token.len());

            // For OAuth2, we need to use XOAUTH2 mechanism
            // The credentials format for XOAUTH2 is: email as username, access_token as password
            let creds = Credentials::new(config.email.clone(), access_token);

            // Use starttls_relay for port 587 (STARTTLS), relay for port 465 (implicit TLS)
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map_err(|e| e.to_string())?
                .credentials(creds)
                .authentication(vec![Mechanism::Xoauth2])
                .build()
        }
        _ => {
            let password = config
                .password
                .clone()
                .ok_or("Password is required for basic authentication")?;

            let creds = Credentials::new(config.email.clone(), password);

            // Use starttls_relay for most SMTP servers
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map_err(|e| e.to_string())?
                .credentials(creds)
                .build()
        }
    };

    tokio::spawn(async move {
        if let Err(e) = mailer.send(email).await {
            eprintln!("Could not send email: {:?}", e);
        } else {
            println!("Email sent successfully!");
        }
    });

    Ok("Started sending email.".into())
}

#[command]
pub async fn reply_email(
    config: AccountConfig,
    to: String,
    original_subject: String,
    body: String,
    attachments: Option<Vec<AttachmentData>>,
) -> Result<String, String> {
    println!("Replying to email: {}", to);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    let from: Mailbox = config.email.parse::<Mailbox>().map_err(|e| e.to_string())?;
    let to_mailbox: Mailbox = to.parse::<Mailbox>().map_err(|e| e.to_string())?;

    // Add "Re: " prefix to subject if not already present
    let reply_subject = if original_subject.to_lowercase().starts_with("re:") {
        original_subject
    } else {
        format!("Re: {}", original_subject)
    };

    let email_builder = Message::builder()
        .from(from)
        .to(to_mailbox)
        .subject(reply_subject);

    // Validate attachment sizes if attachments are present
    if let Some(ref attachment_list) = attachments {
        if !attachment_list.is_empty() {
            validate_attachment_sizes(&config.email, attachment_list)?;
        }
    }

    // Build multipart message if there are attachments
    let email = if let Some(attachment_list) = attachments {
        if !attachment_list.is_empty() {
            let mut multipart = MultiPart::mixed().singlepart(SinglePart::html(body));

            for attachment_data in attachment_list {
                let attachment_body = Body::new(attachment_data.data);
                let attachment = LettreAttachment::new(attachment_data.filename).body(
                    attachment_body,
                    attachment_data.content_type.parse().unwrap(),
                );
                multipart = multipart.singlepart(attachment);
            }

            email_builder
                .multipart(multipart)
                .map_err(|e| e.to_string())?
        } else {
            // Send as HTML even without attachments
            email_builder
                .multipart(MultiPart::alternative().singlepart(SinglePart::html(body)))
                .map_err(|e| e.to_string())?
        }
    } else {
        // Send as HTML even without attachments
        email_builder
            .multipart(MultiPart::alternative().singlepart(SinglePart::html(body)))
            .map_err(|e| e.to_string())?
    };

    let mailer = match config.auth_type {
        Some(AuthType::OAuth2) => {
            let access_token = config
                .access_token
                .clone()
                .ok_or("Access token is required for OAuth2 authentication")?;

            println!(
                "🔐 Building SMTP transport with XOAUTH2 for {}",
                config.email
            );
            println!("   Server: {}:{}", config.smtp_server, config.smtp_port);
            println!("   Token length: {} chars", access_token.len());

            let creds = Credentials::new(config.email.clone(), access_token);

            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map_err(|e| e.to_string())?
                .credentials(creds)
                .authentication(vec![Mechanism::Xoauth2])
                .build()
        }
        _ => {
            let password = config
                .password
                .clone()
                .ok_or("Password is required for basic authentication")?;

            let creds = Credentials::new(config.email.clone(), password);

            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map_err(|e| e.to_string())?
                .credentials(creds)
                .build()
        }
    };

    tokio::spawn(async move {
        if let Err(e) = mailer.send(email).await {
            eprintln!("Could not send reply email: {:?}", e);
        } else {
            println!("Reply email sent successfully!");
        }
    });

    Ok("Started sending reply email.".into())
}

#[command]
pub async fn forward_email(
    config: AccountConfig,
    to: String,
    original_subject: String,
    original_from: String,
    original_to: String,
    original_date: String,
    original_body: String,
    additional_message: String,
    attachments: Option<Vec<AttachmentData>>,
) -> Result<String, String> {
    println!("Forwarding email to: {}", to);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    let from: Mailbox = config.email.parse::<Mailbox>().map_err(|e| e.to_string())?;
    let to_mailbox: Mailbox = to.parse::<Mailbox>().map_err(|e| e.to_string())?;

    // Add "Fwd: " prefix to subject if not already present
    let forward_subject = if original_subject.to_lowercase().starts_with("fwd:") {
        original_subject.clone()
    } else {
        format!("Fwd: {}", original_subject)
    };

    // Build HTML email body with forwarded message metadata and original HTML content
    let forwarded_header = format!(
        r#"<div style="border-top: 1px solid #ccc; margin-top: 20px; padding-top: 10px;">
<p style="font-weight: bold; margin-bottom: 10px;">---------- Forwarded message ----------</p>
<p style="margin: 5px 0;"><strong>From:</strong> {}</p>
<p style="margin: 5px 0;"><strong>To:</strong> {}</p>
<p style="margin: 5px 0;"><strong>Date:</strong> {}</p>
<p style="margin: 5px 0;"><strong>Subject:</strong> {}</p>
</div>
<br/>"#,
        original_from, original_to, original_date, original_subject
    );

    // Strip outer <html><body> tags from original body if present to avoid nesting
    let cleaned_original_body = original_body
        .trim()
        .strip_prefix("<html>")
        .and_then(|s| s.strip_suffix("</html>"))
        .and_then(|s| s.trim().strip_prefix("<body>"))
        .and_then(|s| s.strip_suffix("</body>"))
        .unwrap_or(&original_body)
        .trim();

    // Combine additional message with forwarded content
    let combined_body = if additional_message.is_empty() {
        format!(
            r#"<html><body>{}{}</body></html>"#,
            forwarded_header, cleaned_original_body
        )
    } else {
        // Convert plain text additional message to HTML (replace newlines with <br/>)
        let html_message = additional_message.replace('\n', "<br/>");
        format!(
            r#"<html><body><p>{}</p>{}{}</body></html>"#,
            html_message, forwarded_header, cleaned_original_body
        )
    };

    let email_builder = Message::builder()
        .from(from)
        .to(to_mailbox)
        .subject(forward_subject);

    // Validate attachment sizes if attachments are present
    if let Some(ref attachment_list) = attachments {
        if !attachment_list.is_empty() {
            validate_attachment_sizes(&config.email, attachment_list)?;
        }
    }

    // Build multipart message with attachments if present
    let email = if let Some(attachment_list) = attachments {
        if !attachment_list.is_empty() {
            let mut multipart = MultiPart::mixed().singlepart(SinglePart::html(combined_body));

            for attachment_data in attachment_list {
                let attachment_body = Body::new(attachment_data.data);
                let attachment = LettreAttachment::new(attachment_data.filename).body(
                    attachment_body,
                    attachment_data.content_type.parse().unwrap(),
                );
                multipart = multipart.singlepart(attachment);
            }

            email_builder
                .multipart(multipart)
                .map_err(|e| e.to_string())?
        } else {
            email_builder
                .multipart(MultiPart::alternative().singlepart(SinglePart::html(combined_body)))
                .map_err(|e| e.to_string())?
        }
    } else {
        email_builder
            .multipart(MultiPart::alternative().singlepart(SinglePart::html(combined_body)))
            .map_err(|e| e.to_string())?
    };

    let mailer = match config.auth_type {
        Some(AuthType::OAuth2) => {
            let access_token = config
                .access_token
                .clone()
                .ok_or("Access token is required for OAuth2 authentication")?;

            println!(
                "🔐 Building SMTP transport with XOAUTH2 for {}",
                config.email
            );
            println!("   Server: {}:{}", config.smtp_server, config.smtp_port);
            println!("   Token length: {} chars", access_token.len());

            let creds = Credentials::new(config.email.clone(), access_token);

            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map_err(|e| e.to_string())?
                .credentials(creds)
                .authentication(vec![Mechanism::Xoauth2])
                .build()
        }
        _ => {
            let password = config
                .password
                .clone()
                .ok_or("Password is required for basic authentication")?;

            let creds = Credentials::new(config.email.clone(), password);

            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map_err(|e| e.to_string())?
                .credentials(creds)
                .build()
        }
    };

    tokio::spawn(async move {
        if let Err(e) = mailer.send(email).await {
            eprintln!("Could not forward email: {:?}", e);
        } else {
            println!("Email forwarded successfully!");
        }
    });

    Ok("Started forwarding email.".into())
}

#[command]
pub fn get_attachment_size_limit(email: String) -> Result<u64, String> {
    Ok(get_limit_for_email(&email))
}
