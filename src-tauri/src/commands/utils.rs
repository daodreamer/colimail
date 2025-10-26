use crate::models::{AccountConfig, AuthType};
use crate::oauth2_config::OAuth2Provider;
use crate::security;

/// Helper function to ensure we have a valid access token
pub async fn ensure_valid_token(mut config: AccountConfig) -> Result<AccountConfig, String> {
    // Only process OAuth2 accounts
    if !matches!(config.auth_type, Some(AuthType::OAuth2)) {
        return Ok(config);
    }

    // IMPORTANT: Reload token info from keyring to get the latest credentials
    // This is necessary because the config passed from frontend may be stale
    let creds = security::get_credentials(&config.email)?;

    config.access_token = creds.access_token;
    config.refresh_token = creds.refresh_token;
    config.token_expires_at = creds.token_expires_at;

    // Check if token is expired or about to expire (within 5 minutes)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let needs_refresh = config
        .token_expires_at
        .map(|expires_at| {
            let time_until_expiry = expires_at - now;
            println!(
                "🔍 Token check: expires_at={}, now={}, time_until_expiry={}s ({}min)",
                expires_at,
                now,
                time_until_expiry,
                time_until_expiry / 60
            );
            now >= expires_at - 300
        })
        .unwrap_or(true); // If no expiry time, assume we need to refresh

    if !needs_refresh {
        println!("✓ Access token is still valid");
        return Ok(config);
    }

    println!("⟳ Access token expired or expiring soon, refreshing...");

    // Get refresh token
    let refresh_token = config
        .refresh_token
        .as_ref()
        .ok_or("No refresh token available")?;

    // Determine provider based on IMAP server
    let provider_name = if config.imap_server.contains("gmail") {
        "google"
    } else if config.imap_server.contains("outlook") || config.imap_server.contains("office365") {
        "outlook"
    } else {
        // For unknown providers (e.g., GMX), skip token refresh
        // The existing token will be used as-is
        println!(
            "⚠️  Unknown OAuth2 provider for {}, skipping token refresh",
            config.imap_server
        );
        return Ok(config);
    };

    let provider = OAuth2Provider::get_provider(provider_name)?;

    // Refresh the token
    let (new_access_token, new_expires_at) = provider.refresh_access_token(refresh_token).await?;

    println!("✓ Access token refreshed successfully");

    // Update config with new token
    config.access_token = Some(new_access_token.clone());
    config.token_expires_at = new_expires_at;

    // Update keyring with new token
    security::update_credentials(
        &config.email,
        None, // Don't change password
        Some(new_access_token),
        config.refresh_token.clone(),
        new_expires_at,
    )?;

    println!("✓ Token updated in keyring");

    Ok(config)
}
