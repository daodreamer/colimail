// Notification system for IDLE events
// This module handles desktop notifications

use crate::db;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

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
        println!(
            "🔕 Skipping notification for folder '{}' (not inbox)",
            folder_name
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
            let title = if count == 1 {
                "New Email".to_string()
            } else {
                format!("{} New Emails", count)
            };

            let body = format!("From: {}\nSubject: {}", from, subject);

            println!(
                "📬 Sending notification - Title: '{}', From: '{}', Subject: '{}'",
                title, from, subject
            );

            // Use Tauri's native system notification
            let notification = app_handle.notification();

            // Check permission state before sending
            match notification.permission_state() {
                Ok(state) => {
                    println!("🔔 Current notification permission state: {:?}", state);
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to check notification permission state: {}", e);
                }
            }

            match notification.builder().title(&title).body(&body).show() {
                Ok(_) => {
                    println!("✅ System notification sent successfully");
                }
                Err(e) => {
                    eprintln!("❌ Failed to send system notification: {}", e);
                    eprintln!("   Error details: {:?}", e);
                }
            }
        } else {
            println!("⚠️ Notification enabled but no email data found for notification");
        }
    } else {
        println!("🔕 Notifications are disabled in settings");
    }

    // Play notification sound if enabled
    if sound_enabled {
        // Emit event to frontend to play sound
        let _ = app_handle.emit("play-notification-sound", ());
        println!("🔔 Triggered notification sound");
    }
}
