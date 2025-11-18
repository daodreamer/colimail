use super::types::{CMVHHeaders, EmailContent, VerificationResult};
use secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1};
use sha3::{Digest, Keccak256};

/// Get EIP-712 domain separator for CMVH contract
fn get_domain_separator(chain_id: u64, contract_address: &str) -> Vec<u8> {
    use sha3::{Digest, Keccak256};

    // DOMAIN_TYPEHASH = keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)")
    let domain_typehash: [u8; 32] = [
        0x8b, 0x73, 0xc3, 0xc6, 0x9b, 0xb8, 0xfe, 0x3d, 0x51, 0x2e, 0xcc, 0x4c, 0xf7, 0x59, 0xcc,
        0x79, 0x23, 0x9f, 0x7b, 0x17, 0x9b, 0x0f, 0xfa, 0xca, 0xa9, 0xa7, 0x5d, 0x52, 0x2b, 0x39,
        0x40, 0x0f,
    ];

    let name_hash = {
        let mut hasher = Keccak256::new();
        hasher.update(b"CMVHVerifier");
        hasher.finalize()
    };

    let version_hash = {
        let mut hasher = Keccak256::new();
        hasher.update(b"2.0.0");
        hasher.finalize()
    };

    // Parse contract address
    let addr_hex = contract_address.trim_start_matches("0x");
    let addr_bytes = hex::decode(addr_hex).expect("Invalid contract address");

    // Encode domain separator
    let mut encoded = Vec::with_capacity(32 * 4 + 20);
    encoded.extend_from_slice(&domain_typehash);
    encoded.extend_from_slice(&name_hash);
    encoded.extend_from_slice(&version_hash);

    let mut chain_id_bytes = [0u8; 32];
    chain_id_bytes[24..32].copy_from_slice(&chain_id.to_be_bytes());
    encoded.extend_from_slice(&chain_id_bytes);

    let mut addr_padded = [0u8; 32];
    addr_padded[12..32].copy_from_slice(&addr_bytes);
    encoded.extend_from_slice(&addr_padded);

    let mut hasher = Keccak256::new();
    hasher.update(&encoded);
    hasher.finalize().to_vec()
}

/// Verify CMVH signature and recover signer address (EIP-712)
pub fn verify_signature(headers: &CMVHHeaders, content: &EmailContent) -> VerificationResult {
    println!("🔍 Verifying CMVH signature (EIP-712)");
    println!("   Version: {}", headers.version);
    println!("   Subject: {}", content.subject);
    println!("   From: {} → To: {}", content.from, content.to);

    // Parse timestamp
    let timestamp = match headers.timestamp.parse::<u64>() {
        Ok(ts) => ts,
        Err(e) => {
            return VerificationResult {
                is_valid: false,
                signer_address: None,
                ens_name: None,
                timestamp: None,
                chain: None,
                error: Some(format!("Invalid timestamp: {}", e)),
            }
        }
    };

    println!("   Timestamp: {} ({})", timestamp, headers.timestamp);

    // Compute EIP-712 struct hash
    let struct_hash = content.hash_eip712_struct(timestamp);

    // Get domain separator (Arbitrum Sepolia, CMVH contract)
    let chain_id = 421614u64;
    let contract_address = "0x8f7B72f66C3bC42A8ca6207fDAc7ec1a07641F03";
    let domain_separator = get_domain_separator(chain_id, contract_address);

    // Construct EIP-712 digest: keccak256("\x19\x01" || domainSeparator || structHash)
    let mut digest_input = Vec::with_capacity(2 + 32 + 32);
    digest_input.extend_from_slice(&[0x19, 0x01]);
    digest_input.extend_from_slice(&domain_separator);
    digest_input.extend_from_slice(&struct_hash);

    let mut hasher = Keccak256::new();
    hasher.update(&digest_input);
    let message_hash = hasher.finalize().to_vec();

    println!("   Struct hash: 0x{}", hex::encode(&struct_hash));
    println!("   Domain separator: 0x{}", hex::encode(&domain_separator));
    println!("   Digest: 0x{}", hex::encode(&message_hash));

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

    // Create message from the computed hash
    let message = match Message::from_digest_slice(&message_hash) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eip712_struct_hash() {
        let content = EmailContent {
            subject: "Test Email".to_string(),
            from: "alice@example.com".to_string(),
            to: "bob@example.com".to_string(),
            body: "Test body".to_string(),
        };

        let timestamp = 1700000000u64;
        let hash = content.hash_eip712_struct(timestamp);
        assert_eq!(hash.len(), 32); // keccak256 produces 32 bytes

        // Verify consistency - same input should produce same hash
        let hash2 = content.hash_eip712_struct(timestamp);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_domain_separator() {
        let chain_id = 421614u64;
        let contract_address = "0x8f7B72f66C3bC42A8ca6207fDAc7ec1a07641F03";

        let separator = get_domain_separator(chain_id, contract_address);
        assert_eq!(separator.len(), 32); // keccak256 produces 32 bytes

        // Verify consistency
        let separator2 = get_domain_separator(chain_id, contract_address);
        assert_eq!(separator, separator2);
    }
}
