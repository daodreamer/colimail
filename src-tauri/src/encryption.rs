// Email encryption module using AES-256-GCM
// This module provides encryption/decryption for sensitive email data stored locally

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use std::convert::TryInto;
use std::sync::{Arc, Mutex, OnceLock};
use zeroize::Zeroize;

/// Global encryption key storage (cleared when app closes)
static ENCRYPTION_KEY: OnceLock<Arc<Mutex<Option<Vec<u8>>>>> = OnceLock::new();

/// Error type for encryption operations
#[derive(Debug)]
pub enum EncryptionError {
    KeyNotInitialized,
    EncryptionFailed,
    DecryptionFailed,
    InvalidData,
    KeyDerivationFailed,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::KeyNotInitialized => write!(f, "Encryption key not initialized"),
            EncryptionError::EncryptionFailed => write!(f, "Encryption failed"),
            EncryptionError::DecryptionFailed => write!(f, "Decryption failed"),
            EncryptionError::InvalidData => write!(f, "Invalid encrypted data format"),
            EncryptionError::KeyDerivationFailed => write!(f, "Key derivation failed"),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// Initialize the encryption key storage
fn get_key_storage() -> Arc<Mutex<Option<Vec<u8>>>> {
    ENCRYPTION_KEY
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .clone()
}

/// Derive a 256-bit encryption key from a master password using Argon2
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let argon2 = Argon2::default();

    // Convert salt bytes to SaltString format
    let salt_string =
        SaltString::encode_b64(salt).map_err(|_| EncryptionError::KeyDerivationFailed)?;

    // Hash the password with Argon2
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|_| EncryptionError::KeyDerivationFailed)?;

    // Extract the 32-byte key from the hash
    let hash_bytes = password_hash
        .hash
        .ok_or(EncryptionError::KeyDerivationFailed)?;

    Ok(hash_bytes.as_bytes().to_vec())
}

/// Initialize encryption with a master password
/// This should be called when the user logs in or sets up encryption
pub fn init_encryption(password: &str) -> Result<(), EncryptionError> {
    // Generate a random salt (stored in settings table)
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    // Derive key from password
    let key = derive_key_from_password(password, &salt)?;

    // Store key in memory
    let key_storage = get_key_storage();
    let mut key_guard = key_storage.lock().unwrap();
    *key_guard = Some(key);

    println!("✅ Encryption initialized successfully");
    Ok(())
}

/// Unlock encryption with an existing password and salt
/// Used when the app starts and the user enters their master password
pub fn unlock_encryption(password: &str, salt: &[u8]) -> Result<(), EncryptionError> {
    // Derive key from password and stored salt
    let key = derive_key_from_password(password, salt)?;

    // Store key in memory
    let key_storage = get_key_storage();
    let mut key_guard = key_storage.lock().unwrap();
    *key_guard = Some(key);

    println!("✅ Encryption unlocked successfully");
    Ok(())
}

/// Check if encryption is currently unlocked
pub fn is_encryption_unlocked() -> bool {
    let key_storage = get_key_storage();
    let key_guard = key_storage.lock().unwrap();
    key_guard.is_some()
}

/// Lock encryption (clear the key from memory)
/// Should be called when the user locks the app or logs out
pub fn lock_encryption() {
    let key_storage = get_key_storage();
    let mut key_guard = key_storage.lock().unwrap();
    if let Some(ref mut key) = *key_guard {
        key.zeroize(); // Securely zero out the key
    }
    *key_guard = None;
    println!("🔒 Encryption locked");
}

/// Encrypt data using AES-256-GCM
/// Returns base64-encoded string: nonce(12 bytes) + ciphertext
pub fn encrypt(plaintext: &str) -> Result<String, EncryptionError> {
    let key_storage = get_key_storage();
    let key_guard = key_storage.lock().unwrap();
    let key = key_guard
        .as_ref()
        .ok_or(EncryptionError::KeyNotInitialized)?;

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| EncryptionError::EncryptionFailed)?;

    // Generate random nonce (12 bytes for GCM)
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = &nonce_bytes.into();

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| EncryptionError::EncryptionFailed)?;

    // Combine nonce + ciphertext and encode as base64
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(&result))
}

/// Decrypt data encrypted with AES-256-GCM
/// Expects base64-encoded string: nonce(12 bytes) + ciphertext
pub fn decrypt(encrypted_data: &str) -> Result<String, EncryptionError> {
    let key_storage = get_key_storage();
    let key_guard = key_storage.lock().unwrap();
    let key = key_guard
        .as_ref()
        .ok_or(EncryptionError::KeyNotInitialized)?;

    // Decode base64
    let data = BASE64
        .decode(encrypted_data)
        .map_err(|_| EncryptionError::InvalidData)?;

    // Data must be at least 12 bytes (nonce) + 16 bytes (GCM tag)
    if data.len() < 28 {
        return Err(EncryptionError::InvalidData);
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce_array: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| EncryptionError::InvalidData)?;
    let nonce = Nonce::from(nonce_array);

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| EncryptionError::DecryptionFailed)?;

    // Decrypt the data
    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| EncryptionError::DecryptionFailed)?;

    String::from_utf8(plaintext).map_err(|_| EncryptionError::DecryptionFailed)
}

/// Encrypt binary data (for attachments)
/// Returns base64-encoded string: nonce(12 bytes) + ciphertext
pub fn encrypt_bytes(plaintext: &[u8]) -> Result<String, EncryptionError> {
    let key_storage = get_key_storage();
    let key_guard = key_storage.lock().unwrap();
    let key = key_guard
        .as_ref()
        .ok_or(EncryptionError::KeyNotInitialized)?;

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| EncryptionError::EncryptionFailed)?;

    // Generate random nonce (12 bytes for GCM)
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = &nonce_bytes.into();

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| EncryptionError::EncryptionFailed)?;

    // Combine nonce + ciphertext and encode as base64
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(&result))
}

/// Decrypt binary data (for attachments)
/// Expects base64-encoded string: nonce(12 bytes) + ciphertext
pub fn decrypt_bytes(encrypted_data: &str) -> Result<Vec<u8>, EncryptionError> {
    let key_storage = get_key_storage();
    let key_guard = key_storage.lock().unwrap();
    let key = key_guard
        .as_ref()
        .ok_or(EncryptionError::KeyNotInitialized)?;

    // Decode base64
    let data = BASE64
        .decode(encrypted_data)
        .map_err(|_| EncryptionError::InvalidData)?;

    // Data must be at least 12 bytes (nonce) + 16 bytes (GCM tag)
    if data.len() < 28 {
        return Err(EncryptionError::InvalidData);
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce_array: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| EncryptionError::InvalidData)?;
    let nonce = Nonce::from(nonce_array);

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| EncryptionError::DecryptionFailed)?;

    // Decrypt the data
    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| EncryptionError::DecryptionFailed)
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, EncryptionError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| EncryptionError::KeyDerivationFailed)?;

    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        // Initialize encryption with a test password
        init_encryption("test_password_123").unwrap();

        // Test string encryption
        let plaintext = "Hello, World! 你好世界 🌍";
        let encrypted = encrypt(plaintext).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted);

        // Test binary encryption
        let binary_data = vec![0u8, 1, 2, 3, 255, 128, 64];
        let encrypted_bin = encrypt_bytes(&binary_data).unwrap();
        let decrypted_bin = decrypt_bytes(&encrypted_bin).unwrap();
        assert_eq!(binary_data, decrypted_bin);

        // Clean up
        lock_encryption();
    }

    #[test]
    fn test_key_derivation() {
        let password = "my_secure_password";
        let salt = b"1234567890123456"; // 16 bytes

        let key1 = derive_key_from_password(password, salt).unwrap();
        let key2 = derive_key_from_password(password, salt).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32); // 256 bits
    }

    #[test]
    fn test_encryption_locked() {
        // Make sure encryption is locked initially
        lock_encryption();

        // Trying to encrypt without initializing should fail
        let result = encrypt("test");
        assert!(result.is_err());
    }
}
