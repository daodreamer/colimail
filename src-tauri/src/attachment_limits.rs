// Email provider attachment size limits
// Sizes are in bytes

use std::collections::HashMap;

/// Maximum attachment size limits per email provider (in bytes)
pub fn get_attachment_limits() -> HashMap<&'static str, u64> {
    let mut limits = HashMap::new();

    // Gmail: 25 MB per email
    limits.insert("gmail.com", 25 * 1024 * 1024);
    limits.insert("googlemail.com", 25 * 1024 * 1024);

    // Outlook/Hotmail/Live: 20 MB per email
    limits.insert("outlook.com", 20 * 1024 * 1024);
    limits.insert("hotmail.com", 20 * 1024 * 1024);
    limits.insert("live.com", 20 * 1024 * 1024);

    // Yahoo: 25 MB per email
    limits.insert("yahoo.com", 25 * 1024 * 1024);
    limits.insert("yahoo.co.uk", 25 * 1024 * 1024);
    limits.insert("yahoo.ca", 25 * 1024 * 1024);

    // iCloud: 20 MB per email
    limits.insert("icloud.com", 20 * 1024 * 1024);
    limits.insert("me.com", 20 * 1024 * 1024);
    limits.insert("mac.com", 20 * 1024 * 1024);

    // AOL: 25 MB per email
    limits.insert("aol.com", 25 * 1024 * 1024);

    // ProtonMail: 25 MB per email
    limits.insert("protonmail.com", 25 * 1024 * 1024);
    limits.insert("proton.me", 25 * 1024 * 1024);

    // Zoho: 20 MB per email
    limits.insert("zoho.com", 20 * 1024 * 1024);

    // QQ Mail: 50 MB per email
    limits.insert("qq.com", 50 * 1024 * 1024);

    // 163 Mail: 50 MB per email
    limits.insert("163.com", 50 * 1024 * 1024);
    limits.insert("126.com", 50 * 1024 * 1024);
    limits.insert("yeah.net", 50 * 1024 * 1024);

    // Sina Mail: 50 MB per email
    limits.insert("sina.com", 50 * 1024 * 1024);
    limits.insert("sina.cn", 50 * 1024 * 1024);

    limits
}

/// Get attachment size limit for a specific email address
/// Returns the limit in bytes, or a default of 10 MB if provider is unknown
pub fn get_limit_for_email(email: &str) -> u64 {
    let limits = get_attachment_limits();

    // Extract domain from email
    let domain = email.split('@').nth(1).unwrap_or("");

    // Check if we have a specific limit for this domain
    limits.get(domain).copied().unwrap_or(10 * 1024 * 1024) // Default: 10 MB
}

/// Validate attachment sizes against the email provider's limit
/// Returns Ok(()) if valid, or Err with a descriptive message if invalid
pub fn validate_attachment_sizes(
    email: &str,
    attachments: &[crate::commands::send::AttachmentData],
) -> Result<(), String> {
    let limit = get_limit_for_email(email);

    // Calculate total size of all attachments
    let total_size: u64 = attachments.iter().map(|a| a.data.len() as u64).sum();

    if total_size > limit {
        let limit_mb = limit as f64 / (1024.0 * 1024.0);
        let total_mb = total_size as f64 / (1024.0 * 1024.0);
        return Err(format!(
            "Total attachment size ({:.2} MB) exceeds the limit for your email provider ({:.2} MB)",
            total_mb, limit_mb
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmail_limit() {
        assert_eq!(get_limit_for_email("user@gmail.com"), 25 * 1024 * 1024);
    }

    #[test]
    fn test_outlook_limit() {
        assert_eq!(get_limit_for_email("user@outlook.com"), 20 * 1024 * 1024);
    }

    #[test]
    fn test_unknown_provider() {
        assert_eq!(get_limit_for_email("user@unknown.com"), 10 * 1024 * 1024);
    }
}
