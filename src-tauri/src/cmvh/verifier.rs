use super::types::{CMVHHeaders, EmailContent, VerificationResult};
use secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1};
use sha3::{Digest, Keccak256};

/// Canonicalize email content for consistent hashing
/// Format: "subject\nfrom\nto" (body excluded to avoid HTML formatting issues)
pub fn canonicalize_email(content: &EmailContent) -> String {
    format!("{}\n{}\n{}", content.subject, content.from, content.to)
}

/// Hash email content using keccak256
pub fn hash_email(content: &EmailContent) -> Vec<u8> {
    let canonical = canonicalize_email(content);
    let mut hasher = Keccak256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().to_vec()
}

/// Verify CMVH signature and recover signer address
pub fn verify_signature(headers: &CMVHHeaders, content: &EmailContent) -> VerificationResult {
    println!("🔍 Verifying CMVH signature");
    println!("   Subject: {}", content.subject);
    println!("   From: {} → To: {}", content.from, content.to);

    // Compute email hash
    let email_hash = hash_email(content);

    // Parse signature hex string
    let signature_hex = headers.signature.trim_start_matches("0x");
    let signature_bytes = match hex::decode(signature_hex) {
        Ok(b) => b,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                signer_address: None,
                ens_name: None,
                timestamp: None,
                chain: None,
                error: Some(format!("Invalid signature hex encoding: {}", e)),
            }
        }
    };

    // Signature must be 65 bytes (r: 32, s: 32, v: 1)
    if signature_bytes.len() != 65 {
        return VerificationResult {
            is_valid: false,
            signer_address: None,
            ens_name: None,
            timestamp: None,
            chain: None,
            error: Some(format!(
                "Invalid signature length: expected 65 bytes, got {}",
                signature_bytes.len()
            )),
        };
    }

    // Extract r, s, v components
    let r = &signature_bytes[0..32];
    let s = &signature_bytes[32..64];
    let v = signature_bytes[64];

    // Convert v from Ethereum format (27/28) to recovery id (0/1)
    let recovery_id = if v >= 27 { v - 27 } else { v };
    if recovery_id > 1 {
        return VerificationResult {
            is_valid: false,
            signer_address: None,
            ens_name: None,
            timestamp: None,
            chain: None,
            error: Some(format!("Invalid recovery id: {}", recovery_id)),
        };
    }

    // Create compact signature (64 bytes: r + s)
    let mut compact_sig = [0u8; 64];
    compact_sig[0..32].copy_from_slice(r);
    compact_sig[32..64].copy_from_slice(s);

    // Create recoverable signature
    let rec_id = match secp256k1::ecdsa::RecoveryId::from_i32(recovery_id as i32) {
        Ok(id) => id,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                signer_address: None,
                ens_name: None,
                timestamp: None,
                chain: None,
                error: Some(format!("Failed to create recovery id: {}", e)),
            }
        }
    };

    let recoverable_sig = match RecoverableSignature::from_compact(&compact_sig, rec_id) {
        Ok(sig) => sig,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                signer_address: None,
                ens_name: None,
                timestamp: None,
                chain: None,
                error: Some(format!("Failed to parse signature: {}", e)),
            }
        }
    };

    // Create message from email hash directly (without EIP-191 prefix)
    // Must match the signing process which signs the raw hash
    let message = match Message::from_digest_slice(&email_hash) {
        Ok(msg) => msg,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                signer_address: None,
                ens_name: None,
                timestamp: None,
                chain: None,
                error: Some(format!("Failed to create message: {}", e)),
            }
        }
    };

    // Recover public key
    let secp = Secp256k1::new();
    let public_key = match secp.recover_ecdsa(&message, &recoverable_sig) {
        Ok(pk) => pk,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                signer_address: None,
                ens_name: None,
                timestamp: None,
                chain: None,
                error: Some(format!("Failed to recover public key: {}", e)),
            }
        }
    };

    // Derive Ethereum address from public key
    let public_key_bytes = public_key.serialize_uncompressed();
    // Remove the 0x04 prefix (first byte)
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]);
    let hash = hasher.finalize();
    // Take last 20 bytes
    let address_bytes = &hash[12..];
    let recovered_address = format!("0x{}", hex::encode(address_bytes));

    // Compare with claimed address (case-insensitive)
    let claimed_address = headers.address.to_lowercase();
    let is_valid = recovered_address.to_lowercase() == claimed_address;

    if is_valid {
        println!("✅ CMVH verification PASSED: {}", claimed_address);
    } else {
        println!(
            "❌ CMVH verification FAILED: claimed {} ≠ recovered {}",
            claimed_address,
            recovered_address.to_lowercase()
        );
    }

    VerificationResult {
        is_valid,
        signer_address: if is_valid {
            Some(headers.address.clone())
        } else {
            None
        },
        ens_name: headers.ens.clone(),
        timestamp: Some(headers.timestamp.clone()),
        chain: Some(headers.chain.clone()),
        error: if !is_valid {
            Some(format!(
                "Address mismatch: claimed {}, recovered {}",
                claimed_address, recovered_address
            ))
        } else {
            None
        },
    }
}

/// Create Ethereum signed message hash
/// "\x19Ethereum Signed Message:\n" + len(message) + message
/// Note: Currently unused as we verify raw hash signatures for contract compatibility
#[allow(dead_code)]
fn ethereum_message_hash(message_hash: &[u8]) -> [u8; 32] {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message_hash.len());
    let mut hasher = Keccak256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message_hash);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
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
            body: "Hello, this is a test email.".to_string(),
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
    fn test_ethereum_message_hash() {
        let message = b"test message";
        let hash = ethereum_message_hash(message);
        assert_eq!(hash.len(), 32);
    }
}
