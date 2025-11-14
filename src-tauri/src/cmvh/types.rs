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

impl EmailContent {
    /// Canonical representation for CMVH signing/verification
    /// Format: "subject\nfrom\nto" (body excluded to avoid HTML formatting issues)
    /// This must be consistent across signer and verifier to ensure signatures match
    pub fn canonicalize(&self) -> String {
        format!("{}\n{}\n{}", self.subject, self.from, self.to)
    }

    /// Compute keccak256 hash of canonicalized email content
    pub fn hash_keccak256(&self) -> Vec<u8> {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(self.canonicalize().as_bytes());
        hasher.finalize().to_vec()
    }
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

/// CMVH Error types for fine-grained error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum CMVHError {
    /// Invalid private key format or value
    InvalidPrivateKey { message: String },

    /// Signing operation failed
    SigningFailed { message: String },

    /// Failed to connect to SMTP server
    #[serde(rename = "SMTPConnectionFailed")]
    SMTPConnectionFailed {
        server: String,
        port: u16,
        message: String,
    },

    /// SMTP authentication failed
    #[serde(rename = "SMTPAuthFailed")]
    SMTPAuthFailed { message: String },

    /// Network operation timed out
    NetworkTimeout { duration_secs: u64 },

    /// Rate limited by server
    RateLimited { retry_after_secs: u64 },

    /// Invalid email address format
    InvalidEmailAddress { address: String, message: String },

    /// Email building failed
    EmailBuildFailed { message: String },

    /// Invalid attachment
    InvalidAttachment { filename: String, message: String },

    /// Authentication token error
    TokenError { message: String },

    /// Generic error for uncategorized cases
    Unknown { message: String },
}

impl std::fmt::Display for CMVHError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CMVHError::InvalidPrivateKey { message } => {
                write!(f, "Invalid private key: {}", message)
            }
            CMVHError::SigningFailed { message } => write!(f, "Signing failed: {}", message),
            CMVHError::SMTPConnectionFailed {
                server,
                port,
                message,
            } => write!(
                f,
                "SMTP connection failed ({}:{}): {}",
                server, port, message
            ),
            CMVHError::SMTPAuthFailed { message } => {
                write!(f, "SMTP authentication failed: {}", message)
            }
            CMVHError::NetworkTimeout { duration_secs } => {
                write!(f, "Network timeout after {} seconds", duration_secs)
            }
            CMVHError::RateLimited { retry_after_secs } => {
                write!(f, "Rate limited, retry after {} seconds", retry_after_secs)
            }
            CMVHError::InvalidEmailAddress { address, message } => {
                write!(f, "Invalid email address '{}': {}", address, message)
            }
            CMVHError::EmailBuildFailed { message } => {
                write!(f, "Failed to build email: {}", message)
            }
            CMVHError::InvalidAttachment { filename, message } => {
                write!(f, "Invalid attachment '{}': {}", filename, message)
            }
            CMVHError::TokenError { message } => write!(f, "Token error: {}", message),
            CMVHError::Unknown { message } => write!(f, "Unknown error: {}", message),
        }
    }
}

impl std::error::Error for CMVHError {}

/// Result type for CMVH operations
pub type CMVHResult<T> = Result<T, CMVHError>;
