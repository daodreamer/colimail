use crate::attachment_limits::validate_attachment_sizes;
use crate::cmvh::{
    build_raw_email_with_cmvh, sign_email, CMVHError, CMVHHeaders, CMVHResult, EmailContent,
};
use crate::commands::send::AttachmentData;
use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType};
use lettre::{
    transport::smtp::authentication::{Credentials, Mechanism},
    Address, AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use tauri::command;

/// Extract email addresses from a comma-separated list
fn extract_email_addresses(address_list: &str) -> Vec<String> {
    address_list
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            // Extract email from "Name <email@example.com>" format
            if let Some(start) = s.find('<') {
                if let Some(end) = s.find('>') {
                    return s[start + 1..end].to_string();
                }
            }
            s.to_string()
        })
        .collect()
}

/// Send raw email bytes via SMTP using low-level lettre client
async fn send_raw_email_smtp(
    smtp_server: &str,
    smtp_port: u16,
    credentials: Credentials,
    use_oauth2: bool,
    from: &str,
    to_addresses: Vec<String>,
    raw_email: &[u8],
) -> CMVHResult<()> {
    // Determine TLS strategy based on port
    let use_implicit_tls = smtp_port == 465;

    // Build SMTP connection
    let mut transport_builder = if use_implicit_tls {
        // Port 465: Implicit TLS (SMTPS)
        AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_server)
            .map_err(|e| CMVHError::SMTPConnectionFailed {
                server: smtp_server.to_string(),
                port: smtp_port,
                message: format!("Failed to create SMTP transport: {}", e),
            })?
            .port(smtp_port)
    } else {
        // Other ports: STARTTLS
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_server).map_err(|e| {
            CMVHError::SMTPConnectionFailed {
                server: smtp_server.to_string(),
                port: smtp_port,
                message: format!("Failed to create SMTP transport: {}", e),
            }
        })?
    };

    // Set credentials and authentication mechanism
    if use_oauth2 {
        transport_builder = transport_builder
            .credentials(credentials)
            .authentication(vec![Mechanism::Xoauth2]);
    } else {
        transport_builder = transport_builder.credentials(credentials);
    }

    let transport = transport_builder.build();

    // Connect to SMTP server
    println!("📧 Connecting to SMTP server {}:{}", smtp_server, smtp_port);

    // Use the transport's test_connection to establish connection
    match transport.test_connection().await {
        Ok(true) => println!("✅ SMTP connection established"),
        Ok(false) => {
            return Err(CMVHError::SMTPConnectionFailed {
                server: smtp_server.to_string(),
                port: smtp_port,
                message: "Connection test failed".to_string(),
            })
        }
        Err(e) => {
            // Check if it's an authentication error
            let err_msg = e.to_string();
            if err_msg.contains("authentication") || err_msg.contains("535") {
                return Err(CMVHError::SMTPAuthFailed {
                    message: err_msg.clone(),
                });
            }
            return Err(CMVHError::SMTPConnectionFailed {
                server: smtp_server.to_string(),
                port: smtp_port,
                message: err_msg,
            });
        }
    }

    // Extract sender email
    let sender_email = extract_email_addresses(from)
        .first()
        .ok_or(CMVHError::InvalidEmailAddress {
            address: from.to_string(),
            message: "No valid email address found".to_string(),
        })?
        .clone();

    println!(
        "📨 Sending email from {} to {:?}",
        sender_email, to_addresses
    );

    // Parse email addresses into Address objects
    let from_address: Address =
        sender_email
            .parse()
            .map_err(|e| CMVHError::InvalidEmailAddress {
                address: sender_email.clone(),
                message: format!("Parse error: {}", e),
            })?;

    let to_addrs: Vec<Address> = to_addresses
        .iter()
        .map(|addr| {
            addr.parse().map_err(|e| CMVHError::InvalidEmailAddress {
                address: addr.clone(),
                message: format!("Parse error: {}", e),
            })
        })
        .collect::<CMVHResult<Vec<_>>>()?;

    // Build envelope
    use lettre::address::Envelope;
    let envelope =
        Envelope::new(Some(from_address), to_addrs).map_err(|e| CMVHError::EmailBuildFailed {
            message: format!("Failed to build envelope: {}", e),
        })?;

    // Send the raw email using send_raw
    transport
        .send_raw(&envelope, raw_email)
        .await
        .map_err(|e| {
            let err_msg = e.to_string();
            // Check for rate limiting
            if err_msg.contains("rate limit") || err_msg.contains("429") {
                CMVHError::RateLimited {
                    retry_after_secs: 60, // Default retry after 60 seconds
                }
            } else if err_msg.contains("timeout") {
                CMVHError::NetworkTimeout {
                    duration_secs: 30, // Default timeout duration
                }
            } else {
                CMVHError::Unknown {
                    message: format!("Failed to send email via SMTP: {}", err_msg),
                }
            }
        })?;

    println!("✅ Email sent successfully via SMTP");

    Ok(())
}

#[command]
pub async fn send_email_with_cmvh(
    config: AccountConfig,
    to: String,
    subject: String,
    body: String,
    cc: Option<String>,
    attachments: Option<Vec<AttachmentData>>,
    cmvh_headers: CMVHHeaders,
) -> CMVHResult<String> {
    println!("📧 Sending CMVH-signed email to {}", to);

    // Validate attachment sizes if attachments are present
    if let Some(ref attachment_list) = attachments {
        if !attachment_list.is_empty() {
            validate_attachment_sizes(&config.email, attachment_list).map_err(|e| {
                CMVHError::InvalidAttachment {
                    filename: "multiple".to_string(),
                    message: e,
                }
            })?;
        }
    }

    // Prepare attachment data for raw email builder
    let attachments_data: Option<Vec<(String, String, Vec<u8>)>> = attachments.map(|atts| {
        atts.into_iter()
            .map(|att| (att.filename, att.content_type, att.data))
            .collect()
    });

    // Build raw RFC 5322 email with CMVH headers
    let raw_email = build_raw_email_with_cmvh(
        &config.email,
        &to,
        cc.as_deref(),
        &subject,
        &body,
        &cmvh_headers,
        attachments_data.as_deref(),
    )
    .map_err(|e| CMVHError::EmailBuildFailed { message: e })?;

    println!(
        "✅ Built raw email with CMVH headers ({} bytes)",
        raw_email.len()
    );
    println!("   CMVH-Version: {}", cmvh_headers.version);
    println!("   CMVH-Address: {}", cmvh_headers.address);
    println!(
        "   CMVH-Signature: {}...",
        &cmvh_headers.signature[..20.min(cmvh_headers.signature.len())]
    );

    // Delegate to the specialized send_email_smtp command
    // This ensures we use the separated, testable logic
    send_email_smtp(config, raw_email, to, cc).await?;

    Ok(format!(
        "Email sent successfully with CMVH signature ({}...)",
        &cmvh_headers.signature[..16.min(cmvh_headers.signature.len())]
    ))
}

/// Sign email content with CMVH headers (separated responsibility)
/// This command only handles signing, making it reusable for drafts, previews, etc.
#[command]
pub async fn sign_email_cmvh(
    content: EmailContent,
    private_key: String,
) -> CMVHResult<CMVHHeaders> {
    println!("📝 Signing email with CMVH");
    println!("   Subject: {}", content.subject);
    println!("   From: {} → To: {}", content.from, content.to);

    // Sign the email content
    let headers = sign_email(&private_key, &content)?;

    println!("✅ Email signed successfully");
    println!("   Address: {}", headers.address);
    println!(
        "   Signature: {}...",
        &headers.signature[..16.min(headers.signature.len())]
    );

    Ok(headers)
}

/// Send raw email via SMTP (separated responsibility)
/// This command only handles SMTP sending, making it testable and reusable
#[command]
pub async fn send_email_smtp(
    config: AccountConfig,
    raw_email: Vec<u8>,
    to: String,
    cc: Option<String>,
) -> CMVHResult<String> {
    println!("📧 Sending email via SMTP");
    println!("   Server: {}:{}", config.smtp_server, config.smtp_port);
    println!("   Size: {} bytes", raw_email.len());

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config)
        .await
        .map_err(|e| CMVHError::TokenError { message: e })?;

    // Prepare credentials and authentication type
    let (credentials, use_oauth2) = match config.auth_type {
        Some(AuthType::OAuth2) => {
            let access_token = config.access_token.clone().ok_or(CMVHError::TokenError {
                message: "Access token is required for OAuth2 authentication".to_string(),
            })?;

            println!("🔐 Using OAuth2 authentication (XOAUTH2)");
            (Credentials::new(config.email.clone(), access_token), true)
        }
        _ => {
            let password = config.password.clone().ok_or(CMVHError::SMTPAuthFailed {
                message: "Password is required for basic authentication".to_string(),
            })?;

            println!("🔐 Using basic authentication");
            (Credentials::new(config.email.clone(), password), false)
        }
    };

    // Extract recipient addresses
    let mut to_addresses = extract_email_addresses(&to);
    if let Some(ref cc_str) = cc {
        to_addresses.extend(extract_email_addresses(cc_str));
    }

    // Send the raw email via SMTP
    send_raw_email_smtp(
        &config.smtp_server,
        config.smtp_port,
        credentials,
        use_oauth2,
        &config.email,
        to_addresses,
        &raw_email,
    )
    .await?;

    Ok("Email sent successfully".to_string())
}
