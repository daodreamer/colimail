// Attachment management operations
// This module handles loading and downloading email attachments

use crate::db;
use crate::models::{Attachment, AttachmentInfo};
use std::fs::File;
use std::io::Write;
use tauri::command;

/// Load attachment info from cache (without data)
#[command]
pub async fn load_attachments_info(
    account_id: i32,
    folder_name: String,
    uid: u32,
) -> Result<Vec<AttachmentInfo>, String> {
    let pool = db::pool();

    // First get the email_id
    let email_id_result = sqlx::query_as::<_, (i64,)>(
        "SELECT id FROM emails WHERE account_id = ? AND folder_name = ? AND uid = ?",
    )
    .bind(account_id)
    .bind(&folder_name)
    .bind(uid as i64)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to get email id: {}", e))?;

    let email_id = match email_id_result {
        Some((id,)) => id,
        None => return Ok(Vec::new()), // Email not in cache yet
    };

    // Load attachment info
    let rows = sqlx::query_as::<_, (i64, String, String, i64)>(
        "SELECT id, filename, content_type, size FROM attachments WHERE email_id = ?",
    )
    .bind(email_id)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to load attachments: {}", e))?;

    let attachments: Vec<AttachmentInfo> = rows
        .into_iter()
        .map(|(id, filename, content_type, size)| AttachmentInfo {
            id,
            filename,
            content_type,
            size,
        })
        .collect();

    if !attachments.is_empty() {
        println!(
            "✅ Loaded {} attachments for UID {}",
            attachments.len(),
            uid
        );
    }

    Ok(attachments)
}

/// Download a specific attachment (with data)
#[command]
pub async fn download_attachment(attachment_id: i64) -> Result<Attachment, String> {
    let pool = db::pool();

    let row = sqlx::query_as::<_, (String, String, i64, Vec<u8>)>(
        "SELECT filename, content_type, size, data FROM attachments WHERE id = ?",
    )
    .bind(attachment_id)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to load attachment: {}", e))?;

    Ok(Attachment {
        id: Some(attachment_id),
        filename: row.0,
        content_type: row.1,
        size: row.2,
        data: Some(row.3),
    })
}

/// Save attachment to a file path (for direct file system save)
#[command]
pub async fn save_attachment_to_file(attachment_id: i64, file_path: String) -> Result<(), String> {
    let pool = db::pool();

    // Load attachment data from database
    let row = sqlx::query_as::<_, (Vec<u8>,)>("SELECT data FROM attachments WHERE id = ?")
        .bind(attachment_id)
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to load attachment: {}", e))?;

    let data = row.0;

    // Write to file
    let mut file = File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&data)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    println!(
        "✅ Saved attachment ({} bytes) to: {}",
        data.len(),
        file_path
    );
    Ok(())
}
