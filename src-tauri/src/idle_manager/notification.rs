// Notification system for IDLE events
// This module handles desktop notifications

use crate::db;
use crate::encryption::{decrypt, is_encryption_unlocked};
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

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

/// Check notification and sound settings
async fn check_notification_settings() -> (bool, bool) {
    let pool = db::pool();

    let notification_enabled = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM settings WHERE key = 'notification_enabled'",
    )
    .fetch_one(pool.as_ref())
    .await
    .map(|r| r.0 == "true")
    .unwrap_or(true);

    let sound_enabled =
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'sound_enabled'")
            .fetch_one(pool.as_ref())
            .await
            .map(|r| r.0 == "true")
            .unwrap_or(true);

    (notification_enabled, sound_enabled)
}

/// Check if folder is an inbox folder (case-insensitive)
fn is_inbox_folder(folder_name: &str) -> bool {
    let normalized = folder_name.to_lowercase();
    // Common inbox folder names across different email providers
    normalized == "inbox"
        || normalized == "收件箱"
        || normalized == "收件夹"
        || normalized.contains("inbox")
}

/// Send custom toast notification for new emails
pub async fn send_notification(
    app_handle: &AppHandle,
    account_id: i32,
    folder_name: &str,
    count: u32,
) {
    // Only show notification for inbox folders
    if !is_inbox_folder(folder_name) {
        tracing::debug!(
            folder = %folder_name,
            "Skipping notification for non-inbox folder"
        );
        return;
    }

    let (notification_enabled, sound_enabled) = check_notification_settings().await;

    // Fetch latest email info for notification
    let pool = db::pool();
    let latest_email = sqlx::query_as::<_, (String, String)>(
        "SELECT subject, from_addr FROM emails
         WHERE account_id = ? AND folder_name = ?
         ORDER BY timestamp DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(folder_name)
    .fetch_optional(pool.as_ref())
    .await;

    if notification_enabled {
        if let Ok(Some((subject, from))) = latest_email {
            // Decrypt subject if encryption is enabled and unlocked
            let encryption_enabled = is_encryption_enabled().await.unwrap_or(false);
            let decrypted_subject = if encryption_enabled && is_encryption_unlocked() {
                decrypt(&subject).unwrap_or_else(|e| {
                    tracing::warn!(
                        "Failed to decrypt subject for notification: {}. Using placeholder.",
                        e
                    );
                    "[Encrypted Subject]".to_string()
                })
            } else {
                subject
            };

            let title = if count == 1 {
                "New Email".to_string()
            } else {
                format!("{} New Emails", count)
            };

            let body = format!("From: {}\nSubject: {}", from, decrypted_subject);

            tracing::info!(
                title = %title,
                from = %from,
                subject = %decrypted_subject,
                "Sending notification"
            );

            // Use Tauri's native system notification
            let notification = app_handle.notification();

            // Check permission state before sending
            match notification.permission_state() {
                Ok(state) => {
                    tracing::debug!(state = ?state, "Current notification permission state");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to check notification permission state");
                }
            }

            match notification.builder().title(&title).body(&body).show() {
                Ok(_) => {
                    tracing::info!("System notification sent successfully");
                }
                Err(e) => {
                    tracing::error!(error = %e, details = ?e, "Failed to send system notification");
                }
            }
        } else {
            tracing::warn!("Notification enabled but no email data found for notification");
        }
    } else {
        tracing::debug!("Notifications are disabled in settings");
    }

    // Play notification sound if enabled
    if sound_enabled {
        // Emit event to frontend to play sound
        let _ = app_handle.emit("play-notification-sound", ());
        tracing::debug!("Triggered notification sound");
    }
}
