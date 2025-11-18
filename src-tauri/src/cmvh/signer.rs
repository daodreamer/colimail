use super::types::{CMVHError, CMVHHeaders, CMVHResult, EmailContent};
use hex;
use secp256k1::{Message, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Derive Ethereum address from secret key
pub fn derive_address(secret_key: &SecretKey) -> CMVHResult<String> {
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

/// Get EIP-712 domain separator for CMVH contract
/// Matches: keccak256(abi.encode(DOMAIN_TYPEHASH, keccak256("CMVHVerifier"), keccak256("2.0.0"), chainId, verifyingContract))
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

    // Parse contract address (remove 0x prefix)
    let addr_hex = contract_address.trim_start_matches("0x");
    let addr_bytes = hex::decode(addr_hex).expect("Invalid contract address");

    // Encode domain separator
    let mut encoded = Vec::with_capacity(32 * 4 + 20);
    encoded.extend_from_slice(&domain_typehash);
    encoded.extend_from_slice(&name_hash);
    encoded.extend_from_slice(&version_hash);

    // Encode chainId as uint256
    let mut chain_id_bytes = [0u8; 32];
    chain_id_bytes[24..32].copy_from_slice(&chain_id.to_be_bytes());
    encoded.extend_from_slice(&chain_id_bytes);

    // Encode contract address (left-padded to 32 bytes)
    let mut addr_padded = [0u8; 32];
    addr_padded[12..32].copy_from_slice(&addr_bytes);
    encoded.extend_from_slice(&addr_padded);

    let mut hasher = Keccak256::new();
    hasher.update(&encoded);
    hasher.finalize().to_vec()
}

/// Sign email content with CMVH headers using EIP-712
pub fn sign_email(private_key_hex: &str, content: &EmailContent) -> CMVHResult<CMVHHeaders> {
    println!("📝 Signing email with CMVH (EIP-712)");
    println!("   Subject: {}", content.subject);
    println!("   From: {} → To: {}", content.from, content.to);

    // Parse private key
    let private_key_hex = private_key_hex
        .strip_prefix("0x")
        .unwrap_or(private_key_hex);
    let private_key_bytes =
        hex::decode(private_key_hex).map_err(|e| CMVHError::InvalidPrivateKey {
            message: format!("Invalid hex encoding: {}", e),
        })?;

    let secret_key =
        SecretKey::from_slice(&private_key_bytes).map_err(|e| CMVHError::InvalidPrivateKey {
            message: format!("Invalid key data: {}", e),
        })?;

    // Derive Ethereum address
    let address = derive_address(&secret_key)?;

    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| CMVHError::SigningFailed {
            message: format!("Failed to get system timestamp: {}", e),
        })?
        .as_secs();

    // Compute EIP-712 struct hash with timestamp
    let struct_hash = content.hash_eip712_struct(timestamp);

    // Get domain separator (Arbitrum Sepolia chain ID: 421614, contract address)
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
    let digest = hasher.finalize();

    println!("   Timestamp: {}", timestamp);
    println!("   Struct hash: 0x{}", hex::encode(struct_hash));
    println!("   Domain separator: 0x{}", hex::encode(domain_separator));
    println!("   Digest: 0x{}", hex::encode(digest));

    // Sign the EIP-712 digest
    let secp = Secp256k1::new();
    let message = Message::from_digest_slice(&digest).map_err(|e| CMVHError::SigningFailed {
        message: format!("Failed to create message from digest: {}", e),
    })?;

    let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);
    let (recovery_id, signature_bytes) = signature.serialize_compact();

    // Combine signature bytes with recovery id (v = 27 + recovery_id)
    let mut sig_with_v = signature_bytes.to_vec();
    sig_with_v.push(27 + recovery_id.to_i32() as u8);

    // Format signature as hex
    let signature_hex = format!("0x{}", hex::encode(&sig_with_v));

    // Create CMVH headers
    Ok(CMVHHeaders {
        version: "2".to_string(), // Updated to v2 for EIP-712
        address,
        chain: "Arbitrum".to_string(),
        timestamp: timestamp.to_string(),
        hash_algo: "eip712".to_string(), // Updated to reflect EIP-712
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
    fn test_sign_email_eip712() {
        // Test with known Hardhat account #0
        let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

        let content = EmailContent {
            subject: "Test Email".to_string(),
            from: "sender@example.com".to_string(),
            to: "receiver@example.com".to_string(),
            body: "Test body".to_string(),
        };

        let headers = sign_email(private_key, &content).unwrap();

        // Verify EIP-712 headers
        assert_eq!(headers.version, "2");
        assert_eq!(
            headers.address.to_lowercase(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
        assert_eq!(headers.chain, "Arbitrum");
        assert_eq!(headers.hash_algo, "eip712");
        assert!(headers.signature.starts_with("0x"));
        assert_eq!(headers.signature.len(), 132); // 0x + 130 hex chars (65 bytes)

        // Verify timestamp is present and valid
        assert!(headers.timestamp.parse::<u64>().is_ok());
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
