use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Wry};

lazy_static::lazy_static! {
    // Store PKCE verifiers for pending OAuth flows
    static ref OAUTH_SESSIONS: Mutex<HashMap<String, PkceCodeVerifier>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OAuth2Credentials {
    google: ProviderCredentials,
    outlook: ProviderCredentials,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProviderCredentials {
    client_id: String,
    client_secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

// Use a Mutex-wrapped Option to allow for runtime initialization.
static CREDENTIALS: Mutex<Option<OAuth2Credentials>> = Mutex::new(None);

/// Initializes the OAuth2 credentials.
///
/// This function is called at startup to load the credentials from the appropriate source.
/// The loading order is:
/// 1. Tauri application resources (for release builds).
/// 2. Local `oauth2_credentials.json` file (for development).
/// 3. Environment variables (as a final fallback).
pub fn init_credentials(app: &AppHandle<Wry>) {
    let mut creds_guard = CREDENTIALS.lock().unwrap();
    if creds_guard.is_some() {
        return; // Already initialized
    }

    // 1. Try loading from resource path (for release builds)
    if let Ok(path) = app.path().resolve(
        "oauth2_credentials.json",
        tauri::path::BaseDirectory::Resource,
    ) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(creds) = serde_json::from_str::<OAuth2Credentials>(&content) {
                    println!("✓ Loaded OAuth2 credentials from app resources.");
                    *creds_guard = Some(creds);
                    return;
                } else {
                    eprintln!(
                        "⚠ Warning: Found oauth2_credentials.json in resources, but it's invalid."
                    );
                }
            }
        }
    }

    // 2. Try to load from local file (for development)
    if let Ok(content) = fs::read_to_string("oauth2_credentials.json") {
        if let Ok(creds) = serde_json::from_str::<OAuth2Credentials>(&content) {
            println!("✓ Loaded OAuth2 credentials from local oauth2_credentials.json");
            *creds_guard = Some(creds);
            return;
        } else {
            eprintln!("⚠ Warning: Found local oauth2_credentials.json, but it's invalid.");
        }
    }

    println!(
        "ℹ No valid oauth2_credentials.json found, will use environment variables as fallback."
    );
}

pub struct OAuth2Provider {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
}

impl OAuth2Provider {
    pub fn google() -> Self {
        let creds_guard = CREDENTIALS.lock().unwrap();
        let (client_id, client_secret) = if let Some(creds) = &*creds_guard {
            // Use credentials from the initialized storage
            (
                creds.google.client_id.clone(),
                creds.google.client_secret.clone(),
            )
        } else {
            // Fallback to environment variables if not initialized
            (
                std::env::var("GOOGLE_CLIENT_ID").unwrap_or_else(|_| {
                    eprintln!("⚠ Warning: GOOGLE_CLIENT_ID not set");
                    "YOUR_GOOGLE_CLIENT_ID.apps.googleusercontent.com".to_string()
                }),
                std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_else(|_| {
                    eprintln!("⚠ Warning: GOOGLE_CLIENT_SECRET not set");
                    "YOUR_GOOGLE_CLIENT_SECRET".to_string()
                }),
            )
        };

        Self {
            client_id,
            client_secret,
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            scopes: vec![
                "https://mail.google.com/".to_string(),
                "https://www.googleapis.com/auth/userinfo.email".to_string(),
            ],
            imap_server: "imap.gmail.com".to_string(),
            imap_port: 993,
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587, // Use STARTTLS port instead of 465
        }
    }

    pub fn outlook() -> Self {
        let creds_guard = CREDENTIALS.lock().unwrap();
        let (client_id, client_secret) = if let Some(creds) = &*creds_guard {
            // Use credentials from the initialized storage
            (
                creds.outlook.client_id.clone(),
                creds.outlook.client_secret.clone(),
            )
        } else {
            // Fallback to environment variables if not initialized
            (
                std::env::var("OUTLOOK_CLIENT_ID").unwrap_or_else(|_| {
                    eprintln!("⚠ Warning: OUTLOOK_CLIENT_ID not set");
                    "YOUR_OUTLOOK_CLIENT_ID".to_string()
                }),
                std::env::var("OUTLOOK_CLIENT_SECRET").unwrap_or_else(|_| {
                    eprintln!("⚠ Warning: OUTLOOK_CLIENT_SECRET not set");
                    "YOUR_OUTLOOK_CLIENT_SECRET".to_string()
                }),
            )
        };

        Self {
            client_id,
            client_secret,
            auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
            token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
            scopes: vec![
                // Use Outlook-specific scopes for IMAP/SMTP access
                // Note: These require "Office 365 Exchange Online" API permissions in Azure AD
                "https://outlook.office.com/IMAP.AccessAsUser.All".to_string(),
                "https://outlook.office.com/SMTP.Send".to_string(),
                "offline_access".to_string(),
            ],
            imap_server: "outlook.office365.com".to_string(),
            imap_port: 993,
            smtp_server: "smtp.office365.com".to_string(),
            smtp_port: 587,
        }
    }

    pub fn get_provider(provider: &str) -> Result<Self, String> {
        match provider.to_lowercase().as_str() {
            "google" => Ok(Self::google()),
            "outlook" => Ok(Self::outlook()),
            _ => Err(format!("Unsupported provider: {}", provider)),
        }
    }

    pub fn generate_auth_url(&self) -> Result<(String, String), String> {
        // Validate credentials before proceeding
        if self.client_id.starts_with("YOUR_") {
            return Err(
                "OAuth2 credentials not configured. Please set up oauth2_credentials.json or environment variables.".to_string()
            );
        }

        // Create OAuth2 client
        let client = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new(self.auth_url.clone())
                    .map_err(|e| format!("Invalid auth URL: {}", e))?,
            )
            .set_token_uri(
                TokenUrl::new(self.token_url.clone())
                    .map_err(|e| format!("Invalid token URL: {}", e))?,
            )
            .set_redirect_uri(
                RedirectUrl::new("http://localhost:8765/callback".to_string())
                    .map_err(|e| format!("Invalid redirect URL: {}", e))?,
            );

        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate authorization URL
        let mut auth_request = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        // Add scopes
        for scope in &self.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        let (auth_url, csrf_token) = auth_request.url();

        // Store the verifier for later verification
        let state = csrf_token.secret().to_string();
        OAUTH_SESSIONS
            .lock()
            .unwrap()
            .insert(state.clone(), pkce_verifier);

        Ok((auth_url.to_string(), state))
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        state: &str,
    ) -> Result<(String, Option<String>, Option<i64>), String> {
        // Retrieve and remove the stored verifier
        let pkce_verifier = OAUTH_SESSIONS
            .lock()
            .unwrap()
            .remove(state)
            .ok_or_else(|| "Invalid or expired OAuth state".to_string())?;

        // Create OAuth2 client
        let client = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new(self.auth_url.clone())
                    .map_err(|e| format!("Invalid auth URL: {}", e))?,
            )
            .set_token_uri(
                TokenUrl::new(self.token_url.clone())
                    .map_err(|e| format!("Invalid token URL: {}", e))?,
            )
            .set_redirect_uri(
                RedirectUrl::new("http://localhost:8765/callback".to_string())
                    .map_err(|e| format!("Invalid redirect URL: {}", e))?,
            );

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        // Exchange authorization code for token
        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(&http_client)
            .await
            .map_err(|e| format!("Failed to exchange authorization code: {}", e))?;

        let access_token = token_result.access_token().secret().to_string();
        let refresh_token = token_result.refresh_token().map(|t| t.secret().to_string());

        let expires_at = token_result.expires_in().map(|duration| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            now + duration.as_secs() as i64
        });

        Ok((access_token, refresh_token, expires_at))
    }

    /// Refresh an access token using a refresh token
    pub async fn refresh_access_token(
        &self,
        refresh_token_str: &str,
    ) -> Result<(String, Option<i64>), String> {
        // Create OAuth2 client
        let client = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new(self.auth_url.clone())
                    .map_err(|e| format!("Invalid auth URL: {}", e))?,
            )
            .set_token_uri(
                TokenUrl::new(self.token_url.clone())
                    .map_err(|e| format!("Invalid token URL: {}", e))?,
            );

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        // Exchange refresh token for new access token
        let token_result = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token_str.to_string()))
            .request_async(&http_client)
            .await
            .map_err(|e| format!("Failed to refresh access token: {}", e))?;

        let access_token = token_result.access_token().secret().to_string();

        let expires_at = token_result.expires_in().map(|duration| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            now + duration.as_secs() as i64
        });

        Ok((access_token, expires_at))
    }
}
