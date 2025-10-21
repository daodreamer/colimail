use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

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

static CREDENTIALS: Lazy<Option<OAuth2Credentials>> = Lazy::new(|| {
    // Try to load credentials from file (for official builds)
    if let Ok(content) = fs::read_to_string("oauth2_credentials.json") {
        if let Ok(creds) = serde_json::from_str::<OAuth2Credentials>(&content) {
            println!("✓ Loaded OAuth2 credentials from oauth2_credentials.json");
            return Some(creds);
        } else {
            eprintln!("⚠ Warning: oauth2_credentials.json exists but is invalid");
        }
    }

    // Fallback: check if it's in the executable directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let config_path = exe_dir.join("oauth2_credentials.json");
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(creds) = serde_json::from_str::<OAuth2Credentials>(&content) {
                    println!("✓ Loaded OAuth2 credentials from {}", config_path.display());
                    return Some(creds);
                }
            }
        }
    }

    println!("ℹ No oauth2_credentials.json found, will use environment variables");
    None
});

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
        let (client_id, client_secret) = if let Some(creds) = &*CREDENTIALS {
            // Use credentials from config file
            (
                creds.google.client_id.clone(),
                creds.google.client_secret.clone(),
            )
        } else {
            // Fallback to environment variables
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
            smtp_port: 465,
        }
    }

    pub fn outlook() -> Self {
        let (client_id, client_secret) = if let Some(creds) = &*CREDENTIALS {
            // Use credentials from config file
            (
                creds.outlook.client_id.clone(),
                creds.outlook.client_secret.clone(),
            )
        } else {
            // Fallback to environment variables
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
                "https://outlook.office365.com/IMAP.AccessAsUser.All".to_string(),
                "https://outlook.office365.com/SMTP.Send".to_string(),
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

    /// Check if credentials are configured (for UI validation)
    pub fn are_credentials_configured(provider: &str) -> bool {
        let provider_obj = match Self::get_provider(provider) {
            Ok(p) => p,
            Err(_) => return false,
        };

        !provider_obj.client_id.starts_with("YOUR_")
            && !provider_obj.client_secret.starts_with("YOUR_")
    }
}
