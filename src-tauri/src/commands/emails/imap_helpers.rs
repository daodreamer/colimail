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

    Ok(imap_session)
}
