use crate::cmvh::{
    parse_cmvh_headers, validate_cmvh_headers, verify_signature, CMVHHeaders, EmailContent,
    VerificationResult,
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
    let hash = crate::cmvh::hash_email(&content);
    Ok(format!("0x{}", hex::encode(hash)))
}

/// Check if email has CMVH headers
#[command]
pub async fn has_cmvh_headers(raw_headers: String) -> Result<bool, String> {
    Ok(raw_headers.contains("X-CMVH-Version") || raw_headers.contains("x-cmvh-version"))
}
