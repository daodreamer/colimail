// Encryption management commands
// Handles master password setup, unlock, and encryption status

use crate::db;
use crate::encryption::{
    init_encryption, is_encryption_unlocked, lock_encryption, unlock_encryption, verify_password,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Serialize, Deserialize)]
pub struct EncryptionStatus {
    pub enabled: bool,
    pub unlocked: bool,
}

/// Check if encryption is enabled and unlocked
#[command]
pub async fn get_encryption_status() -> Result<EncryptionStatus, String> {
    let pool = db::pool();

    // Check if encryption is enabled in settings
    let enabled_result = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM settings WHERE key = 'encryption_enabled'",
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to check encryption status: {}", e))?;

    let enabled = enabled_result
        .map(|(value,)| value == "true")
        .unwrap_or(false);

    let unlocked = is_encryption_unlocked();

    Ok(EncryptionStatus { enabled, unlocked })
}

/// Enable encryption with a new master password
/// This sets up encryption for the first time
#[command]
pub async fn enable_encryption(password: String) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    let pool = db::pool();

    // Check if encryption is already enabled
    let enabled_result = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM settings WHERE key = 'encryption_enabled'",
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to check encryption status: {}", e))?;

    if let Some((value,)) = enabled_result {
        if value == "true" {
            return Err("Encryption is already enabled".to_string());
        }
    }

    // Generate a random salt
    let mut salt_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut salt_bytes);
    let salt_b64 = BASE64.encode(salt_bytes);

    // Create password hash for verification
    let argon2 = Argon2::default();
    let salt_string =
        SaltString::encode_b64(&salt_bytes).map_err(|e| format!("Salt encoding failed: {}", e))?;
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("Password hashing failed: {}", e))?
        .to_string();

    // Initialize encryption in memory
    init_encryption(&password).map_err(|e| format!("Failed to initialize encryption: {}", e))?;

    // Store encryption settings in database
    sqlx::query("UPDATE settings SET value = 'true' WHERE key = 'encryption_enabled'")
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to enable encryption: {}", e))?;

    sqlx::query("UPDATE settings SET value = ? WHERE key = 'encryption_salt'")
        .bind(&salt_b64)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to save encryption salt: {}", e))?;

    sqlx::query("UPDATE settings SET value = ? WHERE key = 'password_hash'")
        .bind(&password_hash)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to save password hash: {}", e))?;

    println!("✅ Encryption enabled successfully");
    Ok(())
}

/// Disable encryption
/// WARNING: This will not decrypt existing data - use with caution
#[command]
pub async fn disable_encryption(password: String) -> Result<(), String> {
    let pool = db::pool();

    // Verify password first
    let password_hash_result =
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'password_hash'")
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to get password hash: {}", e))?;

    let password_hash = password_hash_result.ok_or("Encryption is not enabled")?.0;

    if !verify_password(&password, &password_hash)
        .map_err(|e| format!("Password verification failed: {}", e))?
    {
        return Err("Invalid password".to_string());
    }

    // Lock encryption
    lock_encryption();

    // Clear encryption settings
    sqlx::query("UPDATE settings SET value = 'false' WHERE key = 'encryption_enabled'")
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to disable encryption: {}", e))?;

    sqlx::query("UPDATE settings SET value = '' WHERE key = 'encryption_salt'")
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to clear encryption salt: {}", e))?;

    sqlx::query("UPDATE settings SET value = '' WHERE key = 'password_hash'")
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to clear password hash: {}", e))?;

    println!("🔓 Encryption disabled");
    Ok(())
}

/// Unlock encryption with master password
/// Called when the app starts or after locking
#[command]
pub async fn unlock_encryption_with_password(password: String) -> Result<(), String> {
    let pool = db::pool();

    // Check if encryption is enabled
    let enabled_result = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM settings WHERE key = 'encryption_enabled'",
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to check encryption status: {}", e))?;

    let enabled = enabled_result
        .map(|(value,)| value == "true")
        .unwrap_or(false);

    if !enabled {
        return Err("Encryption is not enabled".to_string());
    }

    // Get stored salt and password hash
    let salt_result =
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'encryption_salt'")
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to get encryption salt: {}", e))?;

    let salt_b64 = salt_result.ok_or("Encryption salt not found")?.0;
    let salt_bytes = BASE64
        .decode(&salt_b64)
        .map_err(|e| format!("Failed to decode salt: {}", e))?;

    let password_hash_result =
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'password_hash'")
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to get password hash: {}", e))?;

    let password_hash = password_hash_result.ok_or("Password hash not found")?.0;

    // Verify password
    if !verify_password(&password, &password_hash)
        .map_err(|e| format!("Password verification failed: {}", e))?
    {
        return Err("Invalid password".to_string());
    }

    // Unlock encryption
    unlock_encryption(&password, &salt_bytes)
        .map_err(|e| format!("Failed to unlock encryption: {}", e))?;

    println!("🔓 Encryption unlocked successfully");
    Ok(())
}

/// Lock encryption (clear key from memory)
#[command]
pub fn lock_encryption_command() {
    lock_encryption();
}

/// Change master password
#[command]
pub async fn change_master_password(
    old_password: String,
    new_password: String,
) -> Result<(), String> {
    if new_password.len() < 8 {
        return Err("New password must be at least 8 characters long".to_string());
    }

    let pool = db::pool();

    // Verify old password first
    let password_hash_result =
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'password_hash'")
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to get password hash: {}", e))?;

    let old_password_hash = password_hash_result.ok_or("Encryption is not enabled")?.0;

    if !verify_password(&old_password, &old_password_hash)
        .map_err(|e| format!("Password verification failed: {}", e))?
    {
        return Err("Invalid old password".to_string());
    }

    // Generate a new salt
    let mut salt_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut salt_bytes);
    let salt_b64 = BASE64.encode(salt_bytes);

    // Create new password hash
    let argon2 = Argon2::default();
    let salt_string =
        SaltString::encode_b64(&salt_bytes).map_err(|e| format!("Salt encoding failed: {}", e))?;
    let new_password_hash = argon2
        .hash_password(new_password.as_bytes(), &salt_string)
        .map_err(|e| format!("Password hashing failed: {}", e))?
        .to_string();

    // Re-initialize encryption with new password
    // Note: This only changes the key in memory. Existing encrypted data would need to be re-encrypted
    // with the new key for full password change support. For now, we'll just update the password hash.
    unlock_encryption(&new_password, &salt_bytes)
        .map_err(|e| format!("Failed to unlock with new password: {}", e))?;

    // Update database
    sqlx::query("UPDATE settings SET value = ? WHERE key = 'encryption_salt'")
        .bind(&salt_b64)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to save new encryption salt: {}", e))?;

    sqlx::query("UPDATE settings SET value = ? WHERE key = 'password_hash'")
        .bind(&new_password_hash)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to save new password hash: {}", e))?;

    println!("✅ Master password changed successfully");
    println!(
        "⚠️  Note: Existing encrypted data will need to be re-encrypted with the new password"
    );
    Ok(())
}
