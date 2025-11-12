use super::types::{CMVHHeaders, EmailContent};
use hex;
use secp256k1::{Message, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Canonicalize email content for signing (must match JavaScript implementation)
/// NOTE: Only sign metadata (subject, from, to), not body to avoid HTML formatting issues
pub fn canonicalize_email(content: &EmailContent) -> String {
    format!("{}\n{}\n{}", content.subject, content.from, content.to)
}

/// Compute keccak256 hash of email content
pub fn hash_email(content: &EmailContent) -> Vec<u8> {
    let canonical = canonicalize_email(content);
    let mut hasher = Keccak256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().to_vec()
}

/// Derive Ethereum address from secret key
pub fn derive_address(secret_key: &SecretKey) -> Result<String, String> {
    let secp = Secp256k1::new();
    let public_key = secret_key.public_key(&secp);

    // Get uncompressed public key (65 bytes: 0x04 + x + y)
    let public_key_bytes = public_key.serialize_uncompressed();

    // Take last 64 bytes (skip 0x04 prefix) and hash with keccak256
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]);
    let hash = hasher.finalize();

    // Take last 20 bytes as Ethereum address
    let address_bytes = &hash[12..];
    Ok(format!("0x{}", hex::encode(address_bytes)))
}

/// Sign email content with CMVH headers
pub fn sign_email(private_key_hex: &str, content: &EmailContent) -> Result<CMVHHeaders, String> {
    println!("📝 Signing email with CMVH");
    println!("   Subject: {}", content.subject);
    println!("   From: {} → To: {}", content.from, content.to);

    // Parse private key
    let private_key_hex = private_key_hex
        .strip_prefix("0x")
        .unwrap_or(private_key_hex);
    let private_key_bytes =
        hex::decode(private_key_hex).map_err(|e| format!("Invalid private key hex: {}", e))?;

    let secret_key = SecretKey::from_slice(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    // Derive Ethereum address
    let address = derive_address(&secret_key)?;

    // Hash email content
    let email_hash = hash_email(content);

    // Sign the message hash directly (without EIP-191 prefix)
    // The contract's ECDSA.tryRecover expects signatures of raw hashes
    let secp = Secp256k1::new();
    let message = Message::from_digest_slice(&email_hash)
        .map_err(|e| format!("Failed to create message: {}", e))?;

    let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);
    let (recovery_id, signature_bytes) = signature.serialize_compact();

    // Combine signature bytes with recovery id (v = 27 + recovery_id)
    let mut sig_with_v = signature_bytes.to_vec();
    sig_with_v.push(27 + recovery_id.to_i32() as u8);

    // Format signature as hex
    let signature_hex = format!("0x{}", hex::encode(&sig_with_v));

    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_secs();

    // Create CMVH headers
    Ok(CMVHHeaders {
        version: "1".to_string(),
        address,
        chain: "Arbitrum".to_string(),
        timestamp: timestamp.to_string(),
        hash_algo: "keccak256".to_string(),
        signature: signature_hex,
        ens: None,
        reward: None,
        proof_url: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_email() {
        let content = EmailContent {
            subject: "Test Subject".to_string(),
            from: "alice@example.com".to_string(),
            to: "bob@example.com".to_string(),
            body: "Hello, world!".to_string(),
        };

        let canonical = canonicalize_email(&content);
        assert_eq!(
            canonical,
            "Test Subject\nalice@example.com\nbob@example.com"
        );
    }

    #[test]
    fn test_hash_email() {
        let content = EmailContent {
            subject: "Test".to_string(),
            from: "alice@example.com".to_string(),
            to: "bob@example.com".to_string(),
            body: "Hello".to_string(),
        };

        let hash = hash_email(&content);
        assert_eq!(hash.len(), 32); // keccak256 produces 32 bytes
    }

    #[test]
    fn test_derive_address() {
        // Test with known Hardhat account #0
        let private_key_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let private_key_bytes = hex::decode(private_key_hex).unwrap();
        let secret_key = SecretKey::from_slice(&private_key_bytes).unwrap();

        let address = derive_address(&secret_key).unwrap();
        assert_eq!(
            address.to_lowercase(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }

    #[test]
    fn test_sign_email() {
        // Test with known Hardhat account #0
        let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

        let content = EmailContent {
            subject: "Test Email".to_string(),
            from: "sender@example.com".to_string(),
            to: "receiver@example.com".to_string(),
            body: "Test body".to_string(),
        };

        let headers = sign_email(private_key, &content).unwrap();

        assert_eq!(headers.version, "1");
        assert_eq!(
            headers.address.to_lowercase(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
        assert_eq!(headers.chain, "Arbitrum");
        assert_eq!(headers.hash_algo, "keccak256");
        assert!(headers.signature.starts_with("0x"));
        assert_eq!(headers.signature.len(), 132); // 0x + 130 hex chars (65 bytes)
    }
}
