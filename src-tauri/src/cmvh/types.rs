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
