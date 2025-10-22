use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Basic,
    OAuth2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountConfig {
    pub id: Option<i32>,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
    #[serde(default)]
    pub auth_type: Option<AuthType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OAuth2StartRequest {
    pub provider: String, // "google" or "outlook"
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OAuth2StartResponse {
    pub auth_url: String,
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailHeader {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub date: String,
}
