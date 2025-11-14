use crate::cmvh::{
    derive_address, parse_cmvh_headers, sign_email, validate_cmvh_headers, verify_signature,
    CMVHHeaders, EmailContent, VerificationResult,
};
use tauri::command;

/// Parse CMVH headers from raw email headers
#[command]
pub async fn parse_email_cmvh_headers(raw_headers: String) -> Result<CMVHHeaders, String> {
    parse_cmvh_headers(&raw_headers)
}

/// Verify CMVH signature locally
#[command]
pub async fn verify_cmvh_signature(
    headers: CMVHHeaders,
    content: EmailContent,
) -> Result<VerificationResult, String> {
    // Validate headers format first
    validate_cmvh_headers(&headers)?;

    // Verify signature
    Ok(verify_signature(&headers, &content))
}

/// Hash email content (for debugging/testing)
#[command]
pub async fn hash_email_content(content: EmailContent) -> Result<String, String> {
    let hash = content.hash_keccak256();
    Ok(format!("0x{}", hex::encode(hash)))
}

/// Check if email has CMVH headers
#[command]
pub async fn has_cmvh_headers(raw_headers: String) -> Result<bool, String> {
    Ok(raw_headers.contains("X-CMVH-Version") || raw_headers.contains("x-cmvh-version"))
}

/// Sign email content with CMVH headers
#[command]
pub async fn sign_email_with_cmvh(
    private_key: String,
    content: EmailContent,
) -> Result<CMVHHeaders, String> {
    sign_email(&private_key, &content).map_err(|e| e.to_string())
}

/// Derive Ethereum address from private key
#[command]
pub async fn derive_eth_address(private_key: String) -> Result<String, String> {
    use secp256k1::SecretKey;

    let private_key_hex = private_key.strip_prefix("0x").unwrap_or(&private_key);
    let private_key_bytes =
        hex::decode(private_key_hex).map_err(|e| format!("Invalid private key hex: {}", e))?;

    let secret_key = SecretKey::from_slice(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    derive_address(&secret_key).map_err(|e| e.to_string())
}
