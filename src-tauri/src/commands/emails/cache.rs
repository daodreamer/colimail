// Database cache operations for emails
// This module handles storing and retrieving emails from local SQLite database

use crate::db;
use crate::models::{Attachment, EmailHeader};
use chrono::Utc;
use tauri::command;

/// Save emails to database cache
pub async fn save_emails_to_cache(
    account_id: i32,
    folder_name: &str,
    emails: &[EmailHeader],
) -> Result<(), String> {
    let pool = db::pool();
    let current_time = Utc::now().timestamp();

    for email in emails {
        // Use INSERT with ON CONFLICT to preserve cached body
        sqlx::query(
            "INSERT INTO emails
            (account_id, folder_name, uid, subject, from_addr, to_addr, cc_addr, date, timestamp, synced_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account_id, folder_name, uid) DO UPDATE SET
                subject = excluded.subject,
                from_addr = excluded.from_addr,
                to_addr = excluded.to_addr,
                cc_addr = excluded.cc_addr,
                date = excluded.date,
                timestamp = excluded.timestamp,
                synced_at = excluded.synced_at",
        )
        .bind(account_id)
        .bind(folder_name)
        .bind(email.uid as i64)
        .bind(&email.subject)
        .bind(&email.from)
        .bind(&email.to)
        .bind(&email.cc)
        .bind(&email.date)
        .bind(email.timestamp)
        .bind(current_time)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to save email to cache: {}", e))?;
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

    let rows = sqlx::query_as::<_, (i64, String, String, String, Option<String>, String, i64, i64)>(
        "SELECT uid, subject, from_addr, to_addr, cc_addr, date, timestamp, COALESCE(has_attachments, 0)
        FROM emails
        WHERE account_id = ? AND folder_name = ?
        ORDER BY timestamp DESC",
    )
    .bind(account_id)
    .bind(&folder_name)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to load emails from cache: {}", e))?;

    let emails: Vec<EmailHeader> = rows
        .into_iter()
        .map(
            |(uid, subject, from, to, cc, date, timestamp, has_attachments)| EmailHeader {
                uid: uid as u32,
                subject,
                from,
                to,
                cc: cc.unwrap_or_default(),
                date,
                timestamp,
                has_attachments: has_attachments != 0,
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

    sqlx::query("UPDATE emails SET body = ? WHERE account_id = ? AND folder_name = ? AND uid = ?")
        .bind(body)
        .bind(account_id)
        .bind(folder_name)
        .bind(uid as i64)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to save body to cache: {}", e))?;

    println!("✅ Saved body to cache for UID {}", uid);
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

    Ok(result.and_then(|(body,)| body))
}

/// Save attachments to database
pub async fn save_attachments_to_cache(
    email_id: i64,
    attachments: &[Attachment],
) -> Result<(), String> {
    let pool = db::pool();

    for attachment in attachments {
        if let Some(ref data) = attachment.data {
            sqlx::query(
                "INSERT INTO attachments (email_id, filename, content_type, size, data)
                VALUES (?, ?, ?, ?, ?)",
            )
            .bind(email_id)
            .bind(&attachment.filename)
            .bind(&attachment.content_type)
            .bind(attachment.size)
            .bind(data)
            .execute(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to save attachment: {}", e))?;
        }
    }

    println!("✅ Saved {} attachments to cache", attachments.len());
    Ok(())
}
