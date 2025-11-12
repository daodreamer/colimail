use crate::attachment_limits::{get_limit_for_email, validate_attachment_sizes};
use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType};
use lettre::{
    message::{
        Attachment as LettreAttachment, Body, Mailbox, MessageBuilder, MultiPart, SinglePart,
    },
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tauri::command;

/// Add CMVH headers to email builder
/// Note: Due to lettre library limitations with custom headers, CMVH headers are logged but not added to outgoing emails.
/// Users can verify incoming CMVH-signed emails. For sending CMVH-signed emails, use the provided test scripts.
fn add_cmvh_headers(
    builder: MessageBuilder,
    cmvh_headers: Option<serde_json::Value>,
) -> MessageBuilder {
    if let Some(cmvh) = cmvh_headers {
        println!(
            "CMVH headers provided (not added to email due to lettre limitations): {:?}",
            cmvh
        );
        // TODO: Upgrade to lettre version with better custom header support or use alternative SMTP library
    }
    builder
}

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
    cc: Option<String>,
    attachments: Option<Vec<AttachmentData>>,
    cmvh_headers: Option<serde_json::Value>,
) -> Result<String, String> {
    println!("Sending email to {}", to);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Create From mailbox with display name if available
    let from: Mailbox = if let Some(display_name) = &config.display_name {
        if !display_name.trim().is_empty() {
            format!("{} <{}>", display_name, config.email)
                .parse::<Mailbox>()
                .map_err(|e| e.to_string())?
        } else {
            config.email.parse::<Mailbox>().map_err(|e| e.to_string())?
        }
    } else {
        config.email.parse::<Mailbox>().map_err(|e| e.to_string())?
    };
    let to_mailbox: Mailbox = to.parse::<Mailbox>().map_err(|e| e.to_string())?;

    let email_builder = Message::builder()
        .from(from)
        .to(to_mailbox)
        .subject(subject);

    // Add CMVH headers if provided
    let mut email_builder = add_cmvh_headers(email_builder, cmvh_headers);

    // Add CC recipients if provided
    if let Some(cc_str) = cc {
        if !cc_str.trim().is_empty() {
            // Split by comma and parse each CC recipient
            for cc_addr in cc_str.split(',') {
                let cc_addr = cc_addr.trim();
                if !cc_addr.is_empty() {
                    let cc_mailbox: Mailbox =
                        cc_addr.parse::<Mailbox>().map_err(|e| e.to_string())?;
                    email_builder = email_builder.cc(cc_mailbox);
                }
            }
        }
    }

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

            // Choose connection method based on port
            if config.smtp_port == 465 {
                // Port 465: SSL/TLS (implicit TLS)
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)
                    .map_err(|e| e.to_string())?
                    .credentials(creds)
                    .port(config.smtp_port)
                    .authentication(vec![Mechanism::Xoauth2])
                    .build()
            } else {
                // Port 587: STARTTLS
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                    .map_err(|e| e.to_string())?
                    .credentials(creds)
                    .authentication(vec![Mechanism::Xoauth2])
                    .build()
            }
        }
        _ => {
            let password = config
                .password
                .clone()
                .ok_or("Password is required for basic authentication")?;

            let creds = Credentials::new(config.email.clone(), password);

            // Choose connection method based on port
            if config.smtp_port == 465 {
                // Port 465: SSL/TLS (implicit TLS, used by 163.com, QQ, etc.)
                println!("   Using SSL/TLS (implicit TLS) for port 465");
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)
                    .map_err(|e| e.to_string())?
                    .credentials(creds)
                    .port(config.smtp_port)
                    .build()
            } else {
                // Port 587 or others: STARTTLS
                println!("   Using STARTTLS for port {}", config.smtp_port);
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                    .map_err(|e| e.to_string())?
                    .credentials(creds)
                    .build()
            }
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
    cc: Option<String>,
    attachments: Option<Vec<AttachmentData>>,
    cmvh_headers: Option<serde_json::Value>,
) -> Result<String, String> {
    println!("Replying to email: {}", to);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Create From mailbox with display name if available
    let from: Mailbox = if let Some(display_name) = &config.display_name {
        if !display_name.trim().is_empty() {
            format!("{} <{}>", display_name, config.email)
                .parse::<Mailbox>()
                .map_err(|e| e.to_string())?
        } else {
            config.email.parse::<Mailbox>().map_err(|e| e.to_string())?
        }
    } else {
        config.email.parse::<Mailbox>().map_err(|e| e.to_string())?
    };
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

    // Add CMVH headers if provided
    let mut email_builder = add_cmvh_headers(email_builder, cmvh_headers);

    // Add CC recipients if provided
    if let Some(cc_str) = cc {
        if !cc_str.trim().is_empty() {
            // Split by comma and parse each CC recipient
            for cc_addr in cc_str.split(',') {
                let cc_addr = cc_addr.trim();
                if !cc_addr.is_empty() {
                    let cc_mailbox: Mailbox =
                        cc_addr.parse::<Mailbox>().map_err(|e| e.to_string())?;
                    email_builder = email_builder.cc(cc_mailbox);
                }
            }
        }
    }

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

#[derive(serde::Deserialize)]
pub struct ForwardEmailParamsWithCMVH {
    pub to: String,
    pub original_subject: String,
    pub original_from: String,
    pub original_to: String,
    pub original_date: String,
    pub original_body: String,
    pub additional_message: String,
    pub cc: Option<String>,
    pub attachments: Option<Vec<AttachmentData>>,
    pub cmvh_headers: Option<serde_json::Value>,
}

#[command]
pub async fn forward_email(
    config: AccountConfig,
    params: ForwardEmailParamsWithCMVH,
) -> Result<String, String> {
    println!("Forwarding email to: {}", params.to);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Create From mailbox with display name if available
    let from: Mailbox = if let Some(display_name) = &config.display_name {
        if !display_name.trim().is_empty() {
            format!("{} <{}>", display_name, config.email)
                .parse::<Mailbox>()
                .map_err(|e| e.to_string())?
        } else {
            config.email.parse::<Mailbox>().map_err(|e| e.to_string())?
        }
    } else {
        config.email.parse::<Mailbox>().map_err(|e| e.to_string())?
    };
    let to_mailbox: Mailbox = params.to.parse::<Mailbox>().map_err(|e| e.to_string())?;

    // Add "Fwd: " prefix to subject if not already present
    let forward_subject = if params.original_subject.to_lowercase().starts_with("fwd:") {
        params.original_subject.clone()
    } else {
        format!("Fwd: {}", params.original_subject)
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
        params.original_from, params.original_to, params.original_date, params.original_subject
    );

    // Strip outer <html><body> tags from original body if present to avoid nesting
    let cleaned_original_body = params
        .original_body
        .trim()
        .strip_prefix("<html>")
        .and_then(|s| s.strip_suffix("</html>"))
        .and_then(|s| s.trim().strip_prefix("<body>"))
        .and_then(|s| s.strip_suffix("</body>"))
        .unwrap_or(&params.original_body)
        .trim();

    // Combine additional message with forwarded content
    let combined_body = if params.additional_message.is_empty() {
        format!(
            r#"<html><body>{}{}</body></html>"#,
            forwarded_header, cleaned_original_body
        )
    } else {
        // Convert plain text additional message to HTML (replace newlines with <br/>)
        let html_message = params.additional_message.replace('\n', "<br/>");
        format!(
            r#"<html><body><p>{}</p>{}{}</body></html>"#,
            html_message, forwarded_header, cleaned_original_body
        )
    };

    let email_builder = Message::builder()
        .from(from)
        .to(to_mailbox)
        .subject(forward_subject);

    // Add CMVH headers if provided
    let mut email_builder = add_cmvh_headers(email_builder, params.cmvh_headers);

    // Add CC recipients if provided
    if let Some(cc_str) = params.cc {
        if !cc_str.trim().is_empty() {
            // Split by comma and parse each CC recipient
            for cc_addr in cc_str.split(',') {
                let cc_addr = cc_addr.trim();
                if !cc_addr.is_empty() {
                    let cc_mailbox: Mailbox =
                        cc_addr.parse::<Mailbox>().map_err(|e| e.to_string())?;
                    email_builder = email_builder.cc(cc_mailbox);
                }
            }
        }
    }

    // Validate attachment sizes if attachments are present
    if let Some(ref attachment_list) = params.attachments {
        if !attachment_list.is_empty() {
            validate_attachment_sizes(&config.email, attachment_list)?;
        }
    }

    // Build multipart message with attachments if present
    let email = if let Some(attachment_list) = params.attachments {
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
