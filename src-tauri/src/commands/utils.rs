use crate::db::pool;
use crate::models::{AccountConfig, AuthType};
use crate::oauth2_config::OAuth2Provider;

/// Helper function to ensure we have a valid access token
pub async fn ensure_valid_token(mut config: AccountConfig) -> Result<AccountConfig, String> {
    // Only process OAuth2 accounts
    if !matches!(config.auth_type, Some(AuthType::OAuth2)) {
        return Ok(config);
    }

    // IMPORTANT: Reload token info from database to get the latest expiration time
    // This is necessary because the config passed from frontend may be stale
    let db_pool = pool();
    let db_result = sqlx::query_as::<_, (Option<String>, Option<i64>)>(
        "SELECT access_token, token_expires_at FROM accounts WHERE email = ?",
    )
    .bind(&config.email)
    .fetch_optional(&*db_pool)
    .await
    .map_err(|e| format!("Failed to query token info from database: {}", e))?;

    if let Some((db_access_token, db_token_expires_at)) = db_result {
        config.access_token = db_access_token;
        config.token_expires_at = db_token_expires_at;
    }

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
        return Err("Unknown OAuth2 provider".to_string());
    };

    let provider = OAuth2Provider::get_provider(provider_name)?;

    // Refresh the token
    let (new_access_token, new_expires_at) = provider.refresh_access_token(refresh_token).await?;

    println!("✓ Access token refreshed successfully");

    // Update config with new token
    config.access_token = Some(new_access_token.clone());
    config.token_expires_at = new_expires_at;

    // Update database
    let pool = pool();
    sqlx::query("UPDATE accounts SET access_token = ?, token_expires_at = ? WHERE email = ?")
        .bind(&config.access_token)
        .bind(config.token_expires_at)
        .bind(&config.email)
        .execute(&*pool)
        .await
        .map_err(|e| format!("Failed to update token in database: {}", e))?;

    println!("✓ Token updated in database");

    Ok(config)
}
