use super::types::CMVHHeaders;
use base64::{engine::general_purpose, Engine as _};
use quoted_printable::encode_to_str;

/// Validate header name (only allow alphanumeric and hyphen)
fn validate_header_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 76 {
        return Err(format!("Invalid header name length: {}", name));
    }

    for c in name.chars() {
        if !c.is_ascii_alphanumeric() && c != '-' {
            return Err(format!("Invalid character in header name: {}", c));
        }
    }

    Ok(())
}

/// Validate and sanitize header value (remove CR/LF to prevent injection)
fn sanitize_header_value(value: &str) -> Result<String, String> {
    if value.len() > 998 {
        return Err(format!("Header value too long: {} chars", value.len()));
    }

    // Remove any CR/LF to prevent header injection
    let sanitized = value.replace(['\r', '\n'], "");

    Ok(sanitized)
}

/// Build CMVH headers as key-value pairs
pub fn build_cmvh_header_lines(headers: &CMVHHeaders) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();

    // Define header order and mapping
    let header_map: Vec<(&str, String)> = vec![
        ("X-CMVH-Version", headers.version.clone()),
        ("X-CMVH-Address", headers.address.clone()),
        ("X-CMVH-Chain", headers.chain.clone()),
        ("X-CMVH-Timestamp", headers.timestamp.clone()),
        ("X-CMVH-HashAlgo", headers.hash_algo.clone()),
        ("X-CMVH-Signature", headers.signature.clone()),
    ];

    for (name, value) in header_map {
        validate_header_name(name)?;
        let sanitized_value = sanitize_header_value(&value)?;
        lines.push(format!("{}: {}", name, sanitized_value));
    }

    // Optional headers
    if let Some(ens) = &headers.ens {
        validate_header_name("X-CMVH-ENS")?;
        let sanitized = sanitize_header_value(ens)?;
        lines.push(format!("X-CMVH-ENS: {}", sanitized));
    }

    if let Some(reward) = &headers.reward {
        validate_header_name("X-CMVH-Reward")?;
        let sanitized = sanitize_header_value(reward)?;
        lines.push(format!("X-CMVH-Reward: {}", sanitized));
    }

    if let Some(proof_url) = &headers.proof_url {
        validate_header_name("X-CMVH-ProofURL")?;
        let sanitized = sanitize_header_value(proof_url)?;
        lines.push(format!("X-CMVH-ProofURL: {}", sanitized));
    }

    Ok(lines)
}

/// Build raw RFC 5322 email with CMVH headers
pub fn build_raw_email_with_cmvh(
    from: &str,
    to: &str,
    cc: Option<&str>,
    subject: &str,
    body_html: &str,
    cmvh_headers: &CMVHHeaders,
    attachments: Option<&[(String, String, Vec<u8>)]>, // (filename, content_type, data)
) -> Result<Vec<u8>, String> {
    let mut email = String::new();
    let boundary = format!("----=_Part_{}", chrono::Utc::now().timestamp_millis());

    // Standard headers
    email.push_str(&format!("From: {}\r\n", from));
    email.push_str(&format!("To: {}\r\n", to));
    if let Some(cc_str) = cc {
        if !cc_str.trim().is_empty() {
            email.push_str(&format!("Cc: {}\r\n", cc_str));
        }
    }
    email.push_str(&format!("Subject: {}\r\n", subject));
    email.push_str(&format!("Date: {}\r\n", chrono::Utc::now().to_rfc2822()));
    email.push_str("MIME-Version: 1.0\r\n");

    // Inject CMVH headers BEFORE Content-Type
    let cmvh_lines = build_cmvh_header_lines(cmvh_headers)?;
    for line in cmvh_lines {
        email.push_str(&format!("{}\r\n", line));
    }

    // Content-Type (multipart if attachments, else text/html)
    if let Some(atts) = attachments {
        if !atts.is_empty() {
            email.push_str(&format!(
                "Content-Type: multipart/mixed; boundary=\"{}\"\r\n",
                boundary
            ));
            email.push_str("\r\n");

            // Body part
            email.push_str(&format!("--{}\r\n", boundary));
            email.push_str("Content-Type: text/html; charset=utf-8\r\n");
            email.push_str("Content-Transfer-Encoding: quoted-printable\r\n");
            email.push_str("\r\n");
            email.push_str(&encode_to_str(body_html));
            email.push_str("\r\n\r\n");

            // Attachments
            for (filename, content_type, data) in atts {
                email.push_str(&format!("--{}\r\n", boundary));
                email.push_str(&format!("Content-Type: {}\r\n", content_type));
                email.push_str(&format!(
                    "Content-Disposition: attachment; filename=\"{}\"\r\n",
                    filename
                ));
                email.push_str("Content-Transfer-Encoding: base64\r\n");
                email.push_str("\r\n");
                email.push_str(&general_purpose::STANDARD.encode(data));
                email.push_str("\r\n\r\n");
            }

            email.push_str(&format!("--{}--\r\n", boundary));
        } else {
            // No attachments
            email.push_str("Content-Type: text/html; charset=utf-8\r\n");
            email.push_str("Content-Transfer-Encoding: quoted-printable\r\n");
            email.push_str("\r\n");
            email.push_str(&encode_to_str(body_html));
        }
    } else {
        // No attachments
        email.push_str("Content-Type: text/html; charset=utf-8\r\n");
        email.push_str("Content-Transfer-Encoding: quoted-printable\r\n");
        email.push_str("\r\n");
        email.push_str(&encode_to_str(body_html));
    }

    Ok(email.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmvh::types::CMVHHeaders;

    #[test]
    fn test_validate_header_name() {
        assert!(validate_header_name("X-CMVH-Version").is_ok());
        assert!(validate_header_name("Content-Type").is_ok());
        assert!(validate_header_name("X-Custom-123").is_ok());
        assert!(validate_header_name("Invalid:Name").is_err());
        assert!(validate_header_name("Invalid Name").is_err());
        assert!(validate_header_name("").is_err());
    }

    #[test]
    fn test_sanitize_header_value() {
        assert_eq!(
            sanitize_header_value("normal value").unwrap(),
            "normal value"
        );
        assert_eq!(
            sanitize_header_value("value\r\ninjection").unwrap(),
            "valueinjection"
        );
        assert!(sanitize_header_value(&"x".repeat(1000)).is_err());
    }

    #[test]
    fn test_build_cmvh_header_lines() {
        let headers = CMVHHeaders {
            version: "1".to_string(),
            address: "0x1234567890123456789012345678901234567890".to_string(),
            chain: "Arbitrum".to_string(),
            timestamp: "1234567890".to_string(),
            hash_algo: "keccak256".to_string(),
            signature: "0xabcd".to_string(),
            ens: None,
            reward: None,
            proof_url: None,
        };

        let lines = build_cmvh_header_lines(&headers).unwrap();
        assert_eq!(lines.len(), 6);
        assert!(lines[0].starts_with("X-CMVH-Version: 1"));
        assert!(lines[1].contains("0x1234567890"));
    }

    #[test]
    fn test_encode_quoted_printable() {
        // Test the RFC 2045 compliant quoted-printable encoding
        let text = "Hello World!";
        let encoded = encode_to_str(text);
        assert!(encoded.contains("Hello"));
        assert!(encoded.contains("World!"));

        // Test edge cases: trailing whitespace should be encoded
        let text_with_trailing_space = "Hello World  ";
        let encoded = encode_to_str(text_with_trailing_space);
        // Trailing spaces should be encoded as =20
        assert!(encoded.ends_with("=20=20") || encoded.contains("=20"));
    }

    #[test]
    fn test_build_raw_email_simple() {
        let headers = CMVHHeaders {
            version: "1".to_string(),
            address: "0x1234".to_string(),
            chain: "Arbitrum".to_string(),
            timestamp: "123".to_string(),
            hash_algo: "keccak256".to_string(),
            signature: "0xabcd".to_string(),
            ens: None,
            reward: None,
            proof_url: None,
        };

        let raw = build_raw_email_with_cmvh(
            "alice@example.com",
            "bob@example.com",
            None,
            "Test Subject",
            "<p>Hello World</p>",
            &headers,
            None,
        )
        .unwrap();

        let email_str = String::from_utf8_lossy(&raw);
        assert!(email_str.contains("From: alice@example.com"));
        assert!(email_str.contains("To: bob@example.com"));
        assert!(email_str.contains("Subject: Test Subject"));
        assert!(email_str.contains("X-CMVH-Version: 1"));
        assert!(email_str.contains("X-CMVH-Address: 0x1234"));
        assert!(email_str.contains("X-CMVH-Signature: 0xabcd"));
        assert!(email_str.contains("Hello World"));
    }
}
