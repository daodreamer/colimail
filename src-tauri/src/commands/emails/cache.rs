// Database cache operations for emails
// This module handles storing and retrieving emails from local SQLite database

use crate::db;
use crate::encryption::{decrypt, encrypt, encrypt_bytes, is_encryption_unlocked};
use crate::models::{Attachment, EmailHeader};
use chrono::Utc;
use tauri::command;

/// Check if encryption is enabled in database settings
async fn is_encryption_enabled() -> Result<bool, String> {
    let pool = db::pool();
    let result = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM settings WHERE key = 'encryption_enabled'",
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to check encryption status: {}", e))?;

    Ok(result.map(|(value,)| value == "true").unwrap_or(false))
}

/// Save emails to database cache
pub async fn save_emails_to_cache(
    account_id: i32,
    folder_name: &str,
    emails: &[EmailHeader],
) -> Result<(), String> {
    let pool = db::pool();
    let current_time = Utc::now().timestamp();

    // Check if encryption is enabled
    let encryption_enabled = is_encryption_enabled().await?;

    for email in emails {
        // Encrypt subject if encryption is enabled and unlocked
        let subject_to_store = if encryption_enabled && is_encryption_unlocked() {
            encrypt(&email.subject).map_err(|e| format!("Failed to encrypt subject: {}", e))?
        } else {
            email.subject.clone()
        };

        // Use INSERT with ON CONFLICT to preserve cached body
        let result = sqlx::query(
            "INSERT INTO emails
            (account_id, folder_name, uid, subject, from_addr, to_addr, cc_addr, date, timestamp, has_attachments, seen, flagged, synced_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account_id, folder_name, uid) DO UPDATE SET
                subject = excluded.subject,
                from_addr = excluded.from_addr,
                to_addr = excluded.to_addr,
                cc_addr = excluded.cc_addr,
                date = excluded.date,
                timestamp = excluded.timestamp,
                has_attachments = excluded.has_attachments,
                seen = excluded.seen,
                flagged = excluded.flagged,
                synced_at = excluded.synced_at",
        )
        .bind(account_id)
        .bind(folder_name)
        .bind(email.uid as i64)
        .bind(&subject_to_store)
        .bind(&email.from)
        .bind(&email.to)
        .bind(&email.cc)
        .bind(&email.date)
        .bind(email.timestamp)
        .bind(None::<i64>)  // has_attachments: NULL (未检查), 后台任务会填充
        .bind(email.seen as i64)
        .bind(email.flagged as i64)
        .bind(current_time)
        .execute(pool.as_ref())
        .await;

        // Only log if there's an error
        if let Err(e) = result {
            eprintln!("❌ Failed to save email UID {} to cache: {}", email.uid, e);
            return Err(format!(
                "Failed to save email UID {} to cache: {}",
                email.uid, e
            ));
        }
    }

    println!(
        "✅ Saved {} emails to cache for folder {}",
        emails.len(),
        folder_name
    );
    Ok(())
}

/// Load emails from database cache
#[command]
pub async fn load_emails_from_cache(
    account_id: i32,
    folder: Option<String>,
) -> Result<Vec<EmailHeader>, String> {
    let folder_name = folder.unwrap_or_else(|| "INBOX".to_string());
    println!(
        "Loading emails from cache for account {} folder {}",
        account_id, folder_name
    );

    let pool = db::pool();

    let rows = sqlx::query_as::<_, (i64, String, String, String, Option<String>, String, i64, i64, i64, i64)>(
        "SELECT uid, subject, from_addr, to_addr, cc_addr, date, timestamp, COALESCE(has_attachments, 0), COALESCE(seen, 0), COALESCE(flagged, 0)
        FROM emails
        WHERE account_id = ? AND folder_name = ?
        ORDER BY timestamp DESC",
    )
    .bind(account_id)
    .bind(&folder_name)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to load emails from cache: {}", e))?;

    // Check if encryption is enabled
    let encryption_enabled = is_encryption_enabled().await?;

    let emails: Vec<EmailHeader> = rows
        .into_iter()
        .map(
            |(uid, subject, from, to, cc, date, timestamp, has_attachments, seen, flagged)| {
                // Decrypt subject if encryption is enabled and unlocked
                let decrypted_subject = if encryption_enabled && is_encryption_unlocked() {
                    decrypt(&subject).unwrap_or_else(|e| {
                        tracing::warn!(
                            "Failed to decrypt subject for UID {} (might be during re-encryption): {}. Using placeholder.",
                            uid,
                            e
                        );
                        // Return a placeholder subject so user can still see the email in the list
                        format!("[Subject temporarily unavailable - UID {}]", uid)
                    })
                } else {
                    subject
                };

                EmailHeader {
                    uid: uid as u32,
                    subject: decrypted_subject,
                    from,
                    to,
                    cc: cc.unwrap_or_default(),
                    date,
                    timestamp,
                    has_attachments: has_attachments != 0,
                    seen: seen != 0,
                    flagged: flagged != 0,
                }
            },
        )
        .collect();

    println!(
        "✅ Loaded {} emails from cache for folder {}",
        emails.len(),
        folder_name
    );
    Ok(emails)
}

/// Save email body to cache
pub async fn save_email_body_to_cache(
    account_id: i32,
    folder_name: &str,
    uid: u32,
    body: &str,
) -> Result<(), String> {
    let pool = db::pool();

    // Check if encryption is enabled
    let encryption_enabled = is_encryption_enabled().await?;

    // Encrypt body if encryption is enabled and unlocked
    let body_to_store = if encryption_enabled && is_encryption_unlocked() {
        encrypt(body).map_err(|e| format!("Failed to encrypt body: {}", e))?
    } else {
        body.to_string()
    };

    sqlx::query("UPDATE emails SET body = ? WHERE account_id = ? AND folder_name = ? AND uid = ?")
        .bind(&body_to_store)
        .bind(account_id)
        .bind(folder_name)
        .bind(uid as i64)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to save body to cache: {}", e))?;

    println!(
        "✅ Saved body to cache for UID {} (encrypted: {})",
        uid,
        encryption_enabled && is_encryption_unlocked()
    );
    Ok(())
}

/// Load email body from cache
pub async fn load_email_body_from_cache(
    account_id: i32,
    folder_name: &str,
    uid: u32,
) -> Result<Option<String>, String> {
    let pool = db::pool();

    let result = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT body FROM emails WHERE account_id = ? AND folder_name = ? AND uid = ?",
    )
    .bind(account_id)
    .bind(folder_name)
    .bind(uid as i64)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to load body from cache: {}", e))?;

    // Check if encryption is enabled
    let encryption_enabled = is_encryption_enabled().await?;

    // Decrypt body if encryption is enabled and unlocked
    if let Some((Some(body),)) = result {
        if encryption_enabled && is_encryption_unlocked() {
            match decrypt(&body) {
                Ok(decrypted_body) => Ok(Some(decrypted_body)),
                Err(e) => {
                    // Decryption failed - this could happen during password change re-encryption
                    // Return None to trigger a fresh fetch from server
                    tracing::warn!(
                        "Failed to decrypt body for UID {} (might be during re-encryption): {}. Will fetch from server.",
                        uid,
                        e
                    );
                    Ok(None)
                }
            }
        } else {
            Ok(Some(body))
        }
    } else {
        Ok(None)
    }
}

/// Save raw email headers to cache
pub async fn save_raw_headers_to_cache(
    account_id: i32,
    folder_name: &str,
    uid: u32,
    raw_headers: &str,
) -> Result<(), String> {
    let pool = db::pool();

    // Check if encryption is enabled
    let encryption_enabled = is_encryption_enabled().await?;

    // Encrypt headers if encryption is enabled and unlocked
    let headers_to_store = if encryption_enabled && is_encryption_unlocked() {
        encrypt(raw_headers).map_err(|e| format!("Failed to encrypt headers: {}", e))?
    } else {
        raw_headers.to_string()
    };

    sqlx::query(
        "UPDATE emails SET raw_headers = ? WHERE account_id = ? AND folder_name = ? AND uid = ?",
    )
    .bind(&headers_to_store)
    .bind(account_id)
    .bind(folder_name)
    .bind(uid as i64)
    .execute(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to save raw headers to cache: {}", e))?;

    tracing::debug!(
        "✅ Saved raw headers to cache for UID {} (encrypted: {})",
        uid,
        encryption_enabled && is_encryption_unlocked()
    );
    Ok(())
}

/// Load raw email headers from cache
pub async fn load_raw_headers_from_cache(
    account_id: i32,
    folder_name: &str,
    uid: u32,
) -> Result<Option<String>, String> {
    let pool = db::pool();

    let result = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT raw_headers FROM emails WHERE account_id = ? AND folder_name = ? AND uid = ?",
    )
    .bind(account_id)
    .bind(folder_name)
    .bind(uid as i64)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to load raw headers from cache: {}", e))?;

    // Check if encryption is enabled
    let encryption_enabled = is_encryption_enabled().await?;

    // Decrypt headers if encryption is enabled and unlocked
    if let Some((Some(headers),)) = result {
        if encryption_enabled && is_encryption_unlocked() {
            match decrypt(&headers) {
                Ok(decrypted_headers) => Ok(Some(decrypted_headers)),
                Err(e) => {
                    tracing::warn!(
                        "Failed to decrypt headers for UID {} (might be during re-encryption): {}. Will fetch from server.",
                        uid,
                        e
                    );
                    Ok(None)
                }
            }
        } else {
            Ok(Some(headers))
        }
    } else {
        Ok(None)
    }
}

/// Save attachments to database
pub async fn save_attachments_to_cache(
    email_id: i64,
    attachments: &[Attachment],
) -> Result<(), String> {
    let pool = db::pool();

    // Check if encryption is enabled
    let encryption_enabled = is_encryption_enabled().await?;

    for attachment in attachments {
        if let Some(ref data) = attachment.data {
            // Encrypt attachment data if encryption is enabled and unlocked
            let data_to_store = if encryption_enabled && is_encryption_unlocked() {
                // Encrypt and store as base64 string in BLOB field
                let encrypted = encrypt_bytes(data)
                    .map_err(|e| format!("Failed to encrypt attachment: {}", e))?;
                encrypted.as_bytes().to_vec()
            } else {
                data.clone()
            };

            sqlx::query(
                "INSERT INTO attachments (email_id, filename, content_type, size, data)
                VALUES (?, ?, ?, ?, ?)",
            )
            .bind(email_id)
            .bind(&attachment.filename)
            .bind(&attachment.content_type)
            .bind(attachment.size)
            .bind(&data_to_store)
            .execute(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to save attachment: {}", e))?;
        }
    }

    println!(
        "✅ Saved {} attachments to cache (encrypted: {})",
        attachments.len(),
        encryption_enabled && is_encryption_unlocked()
    );
    Ok(())
}
