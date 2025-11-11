use serde::{Deserialize, Serialize};

/// CMVH headers extracted from email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CMVHHeaders {
    pub version: String,
    pub address: String,
    pub chain: String,
    pub timestamp: String,
    pub hash_algo: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ens: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_url: Option<String>,
}

/// Email content for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailContent {
    pub subject: String,
    pub from: String,
    pub to: String,
    pub body: String,
}

/// Result of signature verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ens_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
