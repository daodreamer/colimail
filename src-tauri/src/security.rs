/// Security module for storing sensitive credentials using OS keyring
///
/// This module provides secure storage for passwords and OAuth2 tokens using:
/// - Windows: Windows Credential Manager
/// - macOS: Keychain
/// - Linux: Secret Service (libsecret)
///
/// All sensitive data is encrypted at rest by the operating system.
///
/// To avoid Windows Credential Manager's 2560 char limit, we store each
/// credential field separately instead of as a single JSON blob.
use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "com.colimail.app";
// Windows Credential Manager limit is 2560 bytes in UTF-16
// UTF-16 uses 2 bytes per character, so max ~1280 characters
// Use 1200 to be safe and account for service/account name overhead
const MAX_CREDENTIAL_LENGTH: usize = 1200;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountCredentials {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<i64>,
}

/// Store a potentially long value by splitting it into chunks
fn store_long_value(service: &str, account: &str, value: &str) -> Result<(), String> {
    if value.len() <= MAX_CREDENTIAL_LENGTH {
        // Value fits in one chunk
        let entry = Entry::new(service, account)
            .map_err(|e| format!("Failed to create entry for {}: {}", account, e))?;
        entry
            .set_password(value)
            .map_err(|e| format!("Failed to store {}: {}", account, e))?;
    } else {
        // Value needs to be split into chunks
        // Split on character boundaries to avoid breaking UTF-8
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < value.len() {
            let end = (start + MAX_CREDENTIAL_LENGTH).min(value.len());
            // Find a valid character boundary
            let mut adjusted_end = end;
            while adjusted_end > start && !value.is_char_boundary(adjusted_end) {
                adjusted_end -= 1;
            }
            chunks.push(&value[start..adjusted_end]);
            start = adjusted_end;
        }

        // Store chunk count
        let count_entry = Entry::new(service, &format!("{}:count", account))
            .map_err(|e| format!("Failed to create count entry: {}", e))?;
        count_entry
            .set_password(&chunks.len().to_string())
            .map_err(|e| format!("Failed to store chunk count: {}", e))?;

        // Store each chunk
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_entry = Entry::new(service, &format!("{}:chunk{}", account, i))
                .map_err(|e| format!("Failed to create chunk {} entry: {}", i, e))?;
            chunk_entry
                .set_password(chunk)
                .map_err(|e| format!("Failed to store chunk {} (len={}): {}", i, chunk.len(), e))?;
        }
    }
    Ok(())
}

/// Retrieve a potentially chunked value
fn retrieve_long_value(service: &str, account: &str) -> Result<Option<String>, String> {
    // Try to get chunk count first
    if let Ok(count_entry) = Entry::new(service, &format!("{}:count", account)) {
        if let Ok(count_str) = count_entry.get_password() {
            if let Ok(count) = count_str.parse::<usize>() {
                // Value was stored in chunks
                let mut result = String::new();
                for i in 0..count {
                    let chunk_entry = Entry::new(service, &format!("{}:chunk{}", account, i))
                        .map_err(|e| format!("Failed to create chunk {} entry: {}", i, e))?;
                    let chunk = chunk_entry
                        .get_password()
                        .map_err(|e| format!("Failed to retrieve chunk {}: {}", i, e))?;
                    result.push_str(&chunk);
                }
                return Ok(Some(result));
            }
        }
    }

    // Try single value
    if let Ok(entry) = Entry::new(service, account) {
        if let Ok(value) = entry.get_password() {
            return Ok(Some(value));
        }
    }

    Ok(None)
}

/// Delete a potentially chunked value
fn delete_long_value(service: &str, account: &str) -> Result<(), String> {
    let mut errors = Vec::new();

    // Try to delete chunk count
    if let Ok(count_entry) = Entry::new(service, &format!("{}:count", account)) {
        if let Ok(count_str) = count_entry.get_password() {
            if let Ok(count) = count_str.parse::<usize>() {
                // Delete all chunks
                for i in 0..count {
                    if let Ok(chunk_entry) = Entry::new(service, &format!("{}:chunk{}", account, i)) {
                        if let Err(e) = chunk_entry.delete_credential() {
                            errors.push(format!("chunk{}: {}", i, e));
                        }
                    }
                }
                // Delete count entry
                if let Err(e) = count_entry.delete_credential() {
                    errors.push(format!("count: {}", e));
                }
            }
        }
    }

    // Also try to delete single value (backward compatibility)
    if let Ok(entry) = Entry::new(service, account) {
        let _ = entry.delete_credential(); // Ignore errors for backward compat
    }

    if !errors.is_empty() {
        Err(format!("Failed to delete some chunks: {:?}", errors))
    } else {
        Ok(())
    }
}

/// Store account credentials securely in the OS keyring
/// Stores each field separately to avoid platform size limits
pub fn store_credentials(creds: &AccountCredentials) -> Result<(), String> {
    // Use base64-encoded hash of email as key to avoid long account names
    // Windows Credential Manager has a limit on the total credential size including the account name
    let email_hash = base64_url_safe(&creds.email);

    // Store password if present
    if let Some(ref password) = creds.password {
        store_long_value(SERVICE_NAME, &format!("{}:pwd", email_hash), password)
            .map_err(|e| format!("Failed to store password: {}", e))?;
    }

    // Store access_token if present
    if let Some(ref access_token) = creds.access_token {
        store_long_value(SERVICE_NAME, &format!("{}:at", email_hash), access_token)
            .map_err(|e| format!("Failed to store access_token: {}", e))?;
    }

    // Store refresh_token if present
    if let Some(ref refresh_token) = creds.refresh_token {
        store_long_value(SERVICE_NAME, &format!("{}:rt", email_hash), refresh_token)
            .map_err(|e| format!("Failed to store refresh_token: {}", e))?;
    }

    // Store token_expires_at if present
    if let Some(expires_at) = creds.token_expires_at {
        let entry = Entry::new(SERVICE_NAME, &format!("{}:exp", email_hash))
            .map_err(|e| format!("Failed to create token_expires_at entry: {}", e))?;
        entry
            .set_password(&expires_at.to_string())
            .map_err(|e| format!("Failed to store token_expires_at: {}", e))?;
    }

    // Store email mapping so we can retrieve by hash later
    let entry = Entry::new(SERVICE_NAME, &format!("{}:email", email_hash))
        .map_err(|e| format!("Failed to create email mapping entry: {}", e))?;
    entry
        .set_password(&creds.email)
        .map_err(|e| format!("Failed to store email mapping: {}", e))?;

    Ok(())
}

/// Generate a short, URL-safe identifier from email
fn base64_url_safe(email: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    email.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Use first 8 characters of hex representation for brevity
    format!("{:x}", hash)[..8].to_string()
}

/// Retrieve account credentials from the OS keyring
pub fn get_credentials(email: &str) -> Result<AccountCredentials, String> {
    let email_hash = base64_url_safe(email);
    
    let mut creds = AccountCredentials {
        email: email.to_string(),
        password: None,
        access_token: None,
        refresh_token: None,
        token_expires_at: None,
    };

    // Retrieve password if exists
    creds.password = retrieve_long_value(SERVICE_NAME, &format!("{}:pwd", email_hash))?;

    // Retrieve access_token if exists
    creds.access_token = retrieve_long_value(SERVICE_NAME, &format!("{}:at", email_hash))?;

    // Retrieve refresh_token if exists
    creds.refresh_token = retrieve_long_value(SERVICE_NAME, &format!("{}:rt", email_hash))?;

    // Retrieve token_expires_at if exists
    if let Ok(entry) = Entry::new(SERVICE_NAME, &format!("{}:exp", email_hash)) {
        if let Ok(expires_at_str) = entry.get_password() {
            if let Ok(expires_at) = expires_at_str.parse::<i64>() {
                creds.token_expires_at = Some(expires_at);
            }
        }
    }

    Ok(creds)
}

/// Delete account credentials from the OS keyring
pub fn delete_credentials(email: &str) -> Result<(), String> {
    let email_hash = base64_url_safe(email);

    // Delete password
    let _ = delete_long_value(SERVICE_NAME, &format!("{}:pwd", email_hash));

    // Delete access_token
    let _ = delete_long_value(SERVICE_NAME, &format!("{}:at", email_hash));

    // Delete refresh_token
    let _ = delete_long_value(SERVICE_NAME, &format!("{}:rt", email_hash));

    // Delete token_expires_at
    if let Ok(entry) = Entry::new(SERVICE_NAME, &format!("{}:exp", email_hash)) {
        let _ = entry.delete_credential();
    }

    // Delete email mapping
    if let Ok(entry) = Entry::new(SERVICE_NAME, &format!("{}:email", email_hash)) {
        let _ = entry.delete_credential();
    }

    Ok(())
}

/// Update specific fields of stored credentials
pub fn update_credentials(
    email: &str,
    password: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    token_expires_at: Option<i64>,
) -> Result<(), String> {
    // Try to get existing credentials, or create new ones
    let mut creds = get_credentials(email).unwrap_or_else(|_| AccountCredentials {
        email: email.to_string(),
        password: None,
        access_token: None,
        refresh_token: None,
        token_expires_at: None,
    });

    // Update fields that are provided
    if password.is_some() {
        creds.password = password;
    }
    if access_token.is_some() {
        creds.access_token = access_token;
    }
    if refresh_token.is_some() {
        creds.refresh_token = refresh_token;
    }
    if token_expires_at.is_some() {
        creds.token_expires_at = token_expires_at;
    }

    store_credentials(&creds)
}
