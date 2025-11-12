use super::types::CMVHHeaders;
use std::collections::HashMap;

/// Parse CMVH headers from raw email headers string
pub fn parse_cmvh_headers(raw_headers: &str) -> Result<CMVHHeaders, String> {
    let mut headers_map: HashMap<String, String> = HashMap::new();

    // Parse header lines
    for line in raw_headers.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            if key.starts_with("X-CMVH-") || key.starts_with("x-cmvh-") {
                let normalized_key = format!(
                    "X-CMVH-{}",
                    key.trim_start_matches("X-CMVH-")
                        .trim_start_matches("x-cmvh-")
                );
                headers_map.insert(normalized_key, value.trim().to_string());
            }
        }
    }

    // Extract required fields
    let version = headers_map
        .get("X-CMVH-Version")
        .ok_or("Missing X-CMVH-Version header")?
        .clone();

    let address = headers_map
        .get("X-CMVH-Address")
        .ok_or("Missing X-CMVH-Address header")?
        .clone();

    let chain = headers_map
        .get("X-CMVH-Chain")
        .ok_or("Missing X-CMVH-Chain header")?
        .clone();

    let timestamp = headers_map
        .get("X-CMVH-Timestamp")
        .ok_or("Missing X-CMVH-Timestamp header")?
        .clone();

    let hash_algo = headers_map
        .get("X-CMVH-HashAlgo")
        .ok_or("Missing X-CMVH-HashAlgo header")?
        .clone();

    let signature = headers_map
        .get("X-CMVH-Signature")
        .ok_or("Missing X-CMVH-Signature header")?
        .clone();

    // Extract optional fields
    let ens = headers_map.get("X-CMVH-ENS").cloned();
    let reward = headers_map.get("X-CMVH-Reward").cloned();
    let proof_url = headers_map.get("X-CMVH-ProofURL").cloned();

    Ok(CMVHHeaders {
        version,
        address,
        chain,
        timestamp,
        hash_algo,
        signature,
        ens,
        reward,
        proof_url,
    })
}

/// Validate CMVH headers format
pub fn validate_cmvh_headers(headers: &CMVHHeaders) -> Result<(), String> {
    // Validate version
    if headers.version != "1" {
        return Err(format!("Unsupported CMVH version: {}", headers.version));
    }

    // Validate hash algorithm
    if headers.hash_algo.to_lowercase() != "keccak256" {
        return Err(format!("Unsupported hash algorithm: {}", headers.hash_algo));
    }

    // Validate address format (0x + 40 hex chars)
    let addr_lower = headers.address.to_lowercase();
    if !addr_lower.starts_with("0x") || addr_lower.len() != 42 {
        return Err("Invalid Ethereum address format".to_string());
    }

    // Check if address contains only hex characters after 0x
    if !addr_lower[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Address contains non-hex characters".to_string());
    }

    // Validate signature format (0x + 130 hex chars for 65-byte signature)
    let sig_lower = headers.signature.to_lowercase();
    if !sig_lower.starts_with("0x") || sig_lower.len() != 132 {
        return Err("Invalid signature format (expected 0x + 130 hex chars)".to_string());
    }

    // Check if signature contains only hex characters after 0x
    if !sig_lower[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Signature contains non-hex characters".to_string());
    }

    // Validate timestamp is a valid number
    if headers.timestamp.parse::<u64>().is_err() {
        return Err("Invalid timestamp format".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_headers() {
        let raw = r#"
From: alice@example.com
To: bob@example.com
Subject: Test Email
X-CMVH-Version: 1
X-CMVH-Address: 0x1234567890123456789012345678901234567890
X-CMVH-Chain: Arbitrum
X-CMVH-Timestamp: 1730733600
X-CMVH-HashAlgo: keccak256
X-CMVH-Signature: 0x1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890
X-CMVH-ENS: alice.eth
        "#;

        let result = parse_cmvh_headers(raw);
        assert!(
            result.is_ok(),
            "Failed to parse headers: {:?}",
            result.err()
        );

        let headers = result.unwrap();
        assert_eq!(headers.version, "1");
        assert_eq!(headers.chain, "Arbitrum");
        assert_eq!(headers.ens, Some("alice.eth".to_string()));
    }

    #[test]
    fn test_parse_missing_required_header() {
        let raw = r#"
X-CMVH-Version: 1
X-CMVH-Address: 0x1234567890123456789012345678901234567890
        "#;

        let result = parse_cmvh_headers(raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_headers() {
        let headers = CMVHHeaders {
            version: "1".to_string(),
            address: "0x1234567890123456789012345678901234567890".to_string(),
            chain: "Arbitrum".to_string(),
            timestamp: "1730733600".to_string(),
            hash_algo: "keccak256".to_string(),
            signature: "0x1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890".to_string(),
            ens: None,
            reward: None,
            proof_url: None,
        };

        let result = validate_cmvh_headers(&headers);
        assert!(result.is_ok(), "Validation failed: {:?}", result.err());
    }

    #[test]
    fn test_validate_invalid_version() {
        let headers = CMVHHeaders {
            version: "2".to_string(),
            address: "0x1234567890123456789012345678901234567890".to_string(),
            chain: "Arbitrum".to_string(),
            timestamp: "1730733600".to_string(),
            hash_algo: "keccak256".to_string(),
            signature: "0x1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890".to_string(),
            ens: None,
            reward: None,
            proof_url: None,
        };

        let result = validate_cmvh_headers(&headers);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported CMVH version"));
    }

    #[test]
    fn test_validate_invalid_address_format() {
        let headers = CMVHHeaders {
            version: "1".to_string(),
            address: "0x123".to_string(), // Too short
            chain: "Arbitrum".to_string(),
            timestamp: "1730733600".to_string(),
            hash_algo: "keccak256".to_string(),
            signature: "0x1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890".to_string(),
            ens: None,
            reward: None,
            proof_url: None,
        };

        let result = validate_cmvh_headers(&headers);
        assert!(result.is_err());
    }
}
