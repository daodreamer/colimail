// Note: Some items in this module are reserved for future Phase 3+ features
#![allow(dead_code, unused_imports)]

use sha3::{Digest, Keccak256};
use std::collections::BTreeMap;

/// Canonical input for email signing
#[derive(Debug, Clone)]
pub struct CanonicalInput {
    pub from: String,
    pub to: String,
    pub cc: Option<String>,
    pub subject: String,
    pub timestamp: u64,
    pub body_html: String,
    pub attachments: Vec<AttachmentInfo>,
}

#[derive(Debug, Clone)]
pub struct AttachmentInfo {
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub content_hash: String,
}

/// Normalize HTML content for consistent hashing
pub fn normalize_html(html: &str) -> String {
    let mut normalized = html.to_string();

    // Remove redundant outer tags
    normalized = normalized.trim().to_string();
    if let Some(stripped) = normalized.strip_prefix("<html>") {
        if let Some(stripped) = stripped.strip_suffix("</html>") {
            normalized = stripped.trim().to_string();
        }
    }
    if let Some(stripped) = normalized.strip_prefix("<body>") {
        if let Some(stripped) = stripped.strip_suffix("</body>") {
            normalized = stripped.trim().to_string();
        }
    }

    // Normalize whitespace: collapse multiple spaces to single space
    let re_spaces = regex::Regex::new(r"\s+").unwrap();
    normalized = re_spaces.replace_all(&normalized, " ").to_string();

    // Normalize line endings to \n
    normalized = normalized.replace("\r\n", "\n").replace('\r', "\n");

    // Trim leading/trailing whitespace
    normalized.trim().to_string()
}

/// Build canonical string for email signing
/// Format: From|To|Cc|Subject|Timestamp|BodyHash|AttachmentsHash
#[allow(clippy::vec_init_then_push)]
pub fn build_canonical_string(input: &CanonicalInput) -> String {
    let mut parts = Vec::new();

    // Add basic fields
    parts.push(input.from.clone());
    parts.push(input.to.clone());
    parts.push(input.cc.clone().unwrap_or_default());
    parts.push(input.subject.clone());
    parts.push(input.timestamp.to_string());

    // Normalize and hash body
    let normalized_body = normalize_html(&input.body_html);
    let body_hash = hash_content(&normalized_body);
    parts.push(body_hash);

    // Hash attachments (sorted by filename for determinism)
    let attachments_hash = if input.attachments.is_empty() {
        String::new()
    } else {
        let mut sorted_attachments = input.attachments.clone();
        sorted_attachments.sort_by(|a, b| a.filename.cmp(&b.filename));

        let mut attachment_strings = Vec::new();
        for att in sorted_attachments {
            attachment_strings.push(format!(
                "{}:{}:{}:{}",
                att.filename, att.mime_type, att.size, att.content_hash
            ));
        }
        hash_content(&attachment_strings.join("|"))
    };
    parts.push(attachments_hash);

    // Join with | separator
    parts.join("|")
}

/// Hash content using keccak256
fn hash_content(content: &str) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(content.as_bytes());
    let hash = hasher.finalize();
    format!("0x{}", hex::encode(hash))
}

/// Compute final hash of canonical email
pub fn compute_email_hash(input: &CanonicalInput) -> Vec<u8> {
    let canonical = build_canonical_string(input);
    let mut hasher = Keccak256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().to_vec()
}

/// Hash attachment content
pub fn hash_attachment_content(content: &[u8]) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(content);
    let hash = hasher.finalize();
    format!("0x{}", hex::encode(hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_html() {
        let html = "<html><body>  Hello   World  </body></html>";
        let normalized = normalize_html(html);
        assert_eq!(normalized, "Hello World");
    }

    #[test]
    fn test_normalize_html_with_newlines() {
        let html = "Line1\r\nLine2\rLine3\nLine4";
        let normalized = normalize_html(html);
        assert_eq!(normalized, "Line1 Line2 Line3 Line4");
    }

    #[test]
    fn test_canonical_string_no_attachments() {
        let input = CanonicalInput {
            from: "alice@example.com".to_string(),
            to: "bob@example.com".to_string(),
            cc: None,
            subject: "Test".to_string(),
            timestamp: 1234567890,
            body_html: "Hello".to_string(),
            attachments: vec![],
        };

        let canonical = build_canonical_string(&input);
        assert!(canonical.contains("alice@example.com"));
        assert!(canonical.contains("bob@example.com"));
        assert!(canonical.contains("Test"));
        assert!(canonical.contains("1234567890"));
    }

    #[test]
    fn test_canonical_string_with_attachments() {
        let input = CanonicalInput {
            from: "alice@example.com".to_string(),
            to: "bob@example.com".to_string(),
            cc: Some("cc@example.com".to_string()),
            subject: "Test".to_string(),
            timestamp: 1234567890,
            body_html: "Hello".to_string(),
            attachments: vec![
                AttachmentInfo {
                    filename: "file1.txt".to_string(),
                    mime_type: "text/plain".to_string(),
                    size: 100,
                    content_hash: "0xabcd".to_string(),
                },
                AttachmentInfo {
                    filename: "file2.txt".to_string(),
                    mime_type: "text/plain".to_string(),
                    size: 200,
                    content_hash: "0xef01".to_string(),
                },
            ],
        };

        let canonical = build_canonical_string(&input);
        assert!(canonical.contains("cc@example.com"));
        // Attachments should be included in hash
        assert!(canonical.len() > 100);
    }

    #[test]
    fn test_compute_email_hash() {
        let input = CanonicalInput {
            from: "alice@example.com".to_string(),
            to: "bob@example.com".to_string(),
            cc: None,
            subject: "Test".to_string(),
            timestamp: 1234567890,
            body_html: "Hello".to_string(),
            attachments: vec![],
        };

        let hash = compute_email_hash(&input);
        assert_eq!(hash.len(), 32); // keccak256 produces 32 bytes
    }
}
