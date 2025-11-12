use crate::attachment_limits::validate_attachment_sizes;
use crate::cmvh::{build_raw_email_with_cmvh, CMVHHeaders};
use crate::commands::send::AttachmentData;
use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType};
use lettre::{
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    Address,
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
) -> Result<(), String> {
    // Determine TLS strategy based on port
    let use_implicit_tls = smtp_port == 465;

    // Build SMTP connection
    let mut transport_builder = if use_implicit_tls {
        // Port 465: Implicit TLS (SMTPS)
        AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_server)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
            .port(smtp_port)
    } else {
        // Other ports: STARTTLS
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_server)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
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
        Ok(false) => return Err("SMTP connection test failed".to_string()),
        Err(e) => return Err(format!("Failed to connect to SMTP server: {}", e)),
    }

    // Extract sender email
    let sender_email = extract_email_addresses(from)
        .first()
        .ok_or("Invalid sender address")?
        .clone();

    println!("📨 Sending email from {} to {:?}", sender_email, to_addresses);

    // Parse email addresses into Address objects
    let from_address: Address = sender_email
        .parse()
        .map_err(|e| format!("Invalid sender address: {}", e))?;

    let to_addrs: Vec<Address> = to_addresses
        .iter()
        .map(|addr| {
            addr.parse()
                .map_err(|e| format!("Invalid recipient address {}: {}", addr, e))
        })
        .collect::<Result<Vec<_>, String>>()?;

    // Build envelope
    use lettre::address::Envelope;
    let envelope = Envelope::new(Some(from_address), to_addrs)
        .map_err(|e| format!("Failed to build envelope: {}", e))?;

    // Send the raw email using send_raw
    transport
        .send_raw(&envelope, raw_email)
        .await
        .map_err(|e| format!("Failed to send email via SMTP: {}", e))?;

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
) -> Result<String, String> {
    println!("Sending CMVH-signed email to {}", to);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Validate attachment sizes if attachments are present
    if let Some(ref attachment_list) = attachments {
        if !attachment_list.is_empty() {
            validate_attachment_sizes(&config.email, attachment_list)?;
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
    )?;

    println!("✅ Built raw email with CMVH headers ({} bytes)", raw_email.len());
    println!("   CMVH-Version: {}", cmvh_headers.version);
    println!("   CMVH-Address: {}", cmvh_headers.address);
    println!("   CMVH-Signature: {}...", &cmvh_headers.signature[..20.min(cmvh_headers.signature.len())]);

    // Log first 500 chars of raw email for debugging
    let preview = String::from_utf8_lossy(&raw_email);
    let preview_len = 500.min(preview.len());
    println!("📝 Email preview:\n{}", &preview[..preview_len]);

    // Prepare credentials and authentication type
    let (credentials, use_oauth2) = match config.auth_type {
        Some(AuthType::OAuth2) => {
            let access_token = config
                .access_token
                .clone()
                .ok_or("Access token is required for OAuth2 authentication")?;

            println!("🔐 Using OAuth2 authentication (XOAUTH2)");
            (Credentials::new(config.email.clone(), access_token), true)
        }
        _ => {
            let password = config
                .password
                .clone()
                .ok_or("Password is required for basic authentication")?;

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

    Ok(format!(
        "Email sent successfully with CMVH signature ({}...)",
        &cmvh_headers.signature[..16.min(cmvh_headers.signature.len())]
    ))
}
