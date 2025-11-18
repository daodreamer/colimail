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
    /// Compute EIP-712 struct hash for email content with timestamp
    /// Matches the contract's getEmailStructHash function
    /// Format: keccak256(abi.encode(EMAIL_TYPEHASH, keccak256(subject), keccak256(from), keccak256(to), timestamp))
    pub fn hash_eip712_struct(&self, timestamp: u64) -> Vec<u8> {
        use sha3::{Digest, Keccak256};

        // EMAIL_TYPEHASH = keccak256("Email(string subject,string from,string to,uint256 timestamp)")
        let email_typehash: [u8; 32] = [
            0x1f, 0xfe, 0xa1, 0xc5, 0x91, 0xc3, 0xff, 0xf4, 0xc1, 0xcb, 0x5c, 0x93, 0xd1, 0x35,
            0xfc, 0xc6, 0x69, 0x94, 0x23, 0xf8, 0x36, 0x42, 0x48, 0xdd, 0xdc, 0x08, 0x28, 0x43,
            0xf6, 0x91, 0xfa, 0xba,
        ];

        // Compute keccak256 hash of each string field
        let subject_hash = {
            let mut hasher = Keccak256::new();
            hasher.update(self.subject.as_bytes());
            hasher.finalize()
        };

        let from_hash = {
            let mut hasher = Keccak256::new();
            hasher.update(self.from.as_bytes());
            hasher.finalize()
        };

        let to_hash = {
            let mut hasher = Keccak256::new();
            hasher.update(self.to.as_bytes());
            hasher.finalize()
        };

        // Construct abi.encode equivalent (concatenate all 32-byte values)
        let mut encoded = Vec::with_capacity(32 * 5);
        encoded.extend_from_slice(&email_typehash);
        encoded.extend_from_slice(&subject_hash);
        encoded.extend_from_slice(&from_hash);
        encoded.extend_from_slice(&to_hash);

        // Encode timestamp as uint256 (32 bytes, big-endian)
        let mut timestamp_bytes = [0u8; 32];
        timestamp_bytes[24..32].copy_from_slice(&timestamp.to_be_bytes());
        encoded.extend_from_slice(&timestamp_bytes);

        // Hash the encoded data
        let mut hasher = Keccak256::new();
        hasher.update(&encoded);
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
