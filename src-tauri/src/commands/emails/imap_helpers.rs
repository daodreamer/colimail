// IMAP connection helpers for imap 3.0.0-alpha.15 API
// Reference: https://docs.rs/imap/3.0.0-alpha.15/imap/index.html

use crate::commands::emails::fetch::OAuth2;
use crate::models::{AccountConfig, AuthType};

/// Connect and login to IMAP server using imap 3.0.0 ClientBuilder API
/// Returns authenticated session ready for use
pub fn connect_and_login(
    config: &AccountConfig,
) -> Result<imap::Session<Box<dyn imap::ImapConnection>>, String> {
    let domain = config.imap_server.as_str();
    let port = config.imap_port;
    let email = config.email.as_str();

    println!("🔌 Connecting to {}:{}", domain, port);

    // Use ClientBuilder::new().connect() as per imap 3.0.0 API
    let client = imap::ClientBuilder::new(domain, port)
        .connect()
        .map_err(|e| format!("Failed to connect to IMAP server: {}", e))?;

    println!("✅ Connected successfully");

    // Authenticate based on auth type
    let imap_session = match &config.auth_type {
        Some(AuthType::OAuth2) => {
            let access_token = config
                .access_token
                .as_ref()
                .ok_or("Access token is required for OAuth2 authentication")?;

            println!("🔐 Authenticating with OAuth2 for: {}", email);

            let oauth2 = OAuth2 {
                user: email.to_string(),
                access_token: access_token.to_string(),
            };

            client
                .authenticate("XOAUTH2", &oauth2)
                .map_err(|e| format!("OAuth2 authentication failed: {}", e.0))?
        }
        _ => {
            let password = config
                .password
                .as_ref()
                .ok_or("Password is required for password authentication")?;

            println!("🔐 Authenticating with password for: {}", email);

            client
                .login(email, password)
                .map_err(|e| format!("Login failed: {}", e.0))?
        }
    };

    println!("✅ Authentication successful");

    // Send IMAP ID command after authentication for providers that require it (like 163.com)
    // This identifies the client to the server
    let mut session = imap_session;
    if should_send_imap_id(domain) {
        println!("📧 Sending IMAP ID command for {}", domain);
        if let Err(e) = send_imap_id(&mut session) {
            eprintln!("⚠️ Failed to send IMAP ID: {}", e);
            // Continue anyway, authentication already succeeded
        }
    }

    Ok(session)
}

/// Check if IMAP ID command should be sent for this domain
/// Chinese email providers (163.com, 126.com, qq.com, sina.com) require IMAP ID
fn should_send_imap_id(domain: &str) -> bool {
    let domain_lower = domain.to_lowercase();
    domain_lower.contains("163.com")
        || domain_lower.contains("126.com")
        || domain_lower.contains("yeah.net")
        || domain_lower.contains("qq.com")
        || domain_lower.contains("sina.com")
        || domain_lower.contains("sohu.com")
}

/// Send IMAP ID command to identify the client
/// Required by Chinese email providers like 163.com, 126.com, QQ, etc.
fn send_imap_id(session: &mut imap::Session<Box<dyn imap::ImapConnection>>) -> Result<(), String> {
    // Build ID command manually since imap crate doesn't have a dedicated method
    // Format: ID ("name" "Colimail" "version" "0.4.1" "vendor" "Colimail")
    let id_params = vec![
        ("name", "Colimail"),
        ("version", env!("CARGO_PKG_VERSION")),
        ("vendor", "Colimail"),
        ("support-email", "support@colimail.com"),
    ];

    // Format ID parameters as IMAP LIST
    let mut params_str = String::new();
    for (key, value) in id_params {
        if !params_str.is_empty() {
            params_str.push(' ');
        }
        params_str.push_str(&format!("\"{}\" \"{}\"", key, value));
    }

    // Send raw IMAP command using Session's public run_command_and_read_response method
    let command = format!("ID ({})", params_str);

    match session.run_command_and_read_response(&command) {
        Ok(response) => {
            println!("✅ IMAP ID sent successfully");
            println!("   Server response: {}", String::from_utf8_lossy(&response));
            Ok(())
        }
        Err(e) => Err(format!("Failed to send IMAP ID: {}", e)),
    }
}
