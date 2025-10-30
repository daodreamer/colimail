// Notification system for IDLE events
// This module handles desktop notifications and notification windows

use super::types::{NotificationData, NOTIFICATION_QUEUE, NOTIFICATION_WORKER_RUNNING};
use crate::db;
use std::time::Duration;
use tauri::{AppHandle, Emitter, LogicalPosition, WebviewUrl, WebviewWindowBuilder};

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
                "新邮件".to_string()
            } else {
                format!("{} 封新邮件", count)
            };

            println!(
                "📬 Queuing notification - Title: '{}', From: '{}', Subject: '{}'",
                title, from, subject
            );

            // Add notification to queue
            {
                let mut queue = NOTIFICATION_QUEUE.lock().unwrap();
                queue.push(NotificationData {
                    title: title.clone(),
                    from: from.clone(),
                    subject: subject.clone(),
                });
                println!("📋 Notification queue size: {}", queue.len());
            }

            // Start notification worker if not already running
            start_notification_worker(app_handle.clone());
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

/// Start the notification worker that processes the queue
fn start_notification_worker(app_handle: AppHandle) {
    let mut is_running = NOTIFICATION_WORKER_RUNNING.lock().unwrap();

    if *is_running {
        println!("⏳ Notification worker already running");
        return;
    }

    *is_running = true;
    drop(is_running); // Release lock before spawning thread

    println!("🚀 Starting notification worker");

    std::thread::spawn(move || {
        loop {
            // Get next notification from queue
            let notification = {
                let mut queue = NOTIFICATION_QUEUE.lock().unwrap();
                if queue.is_empty() {
                    // No more notifications, stop worker
                    let mut running = NOTIFICATION_WORKER_RUNNING.lock().unwrap();
                    *running = false;
                    println!("⏹️ Notification worker stopped (queue empty)");
                    break;
                }
                queue.remove(0) // Get first item
            };

            println!("📤 Processing notification: '{}'", notification.title);

            // Create and show notification window
            create_notification_window(
                &app_handle,
                &notification.title,
                &notification.from,
                &notification.subject,
            );

            // Wait 5 seconds before processing next notification
            std::thread::sleep(Duration::from_secs(5));
        }
    });
}

/// Create a notification window that appears on screen
fn create_notification_window(app_handle: &AppHandle, title: &str, from: &str, subject: &str) {
    // Generate unique window label with timestamp
    let window_label = format!(
        "notification-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // URL encode parameters
    let encoded_title = urlencoding::encode(title);
    let encoded_from = urlencoding::encode(from);
    let encoded_subject = urlencoding::encode(subject);

    // Build notification URL
    let url = format!(
        "notification.html?title={}&from={}&subject={}",
        encoded_title, encoded_from, encoded_subject
    );

    // Get screen dimensions to position in bottom-right
    let window_width = 380;
    let window_height = 120;
    let margin_right = 20; // Distance from right edge (comfortable margin)
    let margin_bottom = 20; // Distance from bottom edge (comfortable margin)

    // Create window builder
    match WebviewWindowBuilder::new(app_handle, &window_label, WebviewUrl::App(url.into()))
        .title("通知")
        .inner_size(window_width as f64, window_height as f64)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false) // Start invisible, will show after positioning
        .focused(false)
        .build()
    {
        Ok(window) => {
            // Get primary monitor to calculate position using available work area
            if let Some(monitor) = window.current_monitor().ok().flatten() {
                // Get monitor properties
                let work_size = monitor.size();
                let work_position = monitor.position();
                let scale_factor = monitor.scale_factor();

                // Convert physical pixels to logical pixels for macOS Retina displays
                // On Retina displays, scale_factor is typically 2.0
                let logical_width = (work_size.width as f64 / scale_factor) as i32;
                let logical_height = (work_size.height as f64 / scale_factor) as i32;
                let logical_pos_x = (work_position.x as f64 / scale_factor) as i32;
                let logical_pos_y = (work_position.y as f64 / scale_factor) as i32;

                // Calculate position in logical pixels (bottom-right corner of work area)
                // This ensures correct positioning on both regular and Retina displays
                let x = logical_pos_x + logical_width - window_width - margin_right;
                let y = logical_pos_y + logical_height - window_height - margin_bottom;

                println!(
                    "📍 Monitor - physical size: {:?}, position: {:?}, scale: {}x",
                    work_size, work_position, scale_factor
                );
                println!(
                    "📍 Monitor - logical size: {}x{}, position: ({}, {})",
                    logical_width, logical_height, logical_pos_x, logical_pos_y
                );
                println!("📍 Notification window position: ({}, {})", x, y);
                println!(
                    "📍 Window size - logical: {}x{}, margins: right={}px, bottom={}px",
                    window_width, window_height, margin_right, margin_bottom
                );

                // Set position using LogicalPosition (not PhysicalPosition!)
                // This is crucial for Retina displays where logical != physical pixels
                if let Err(e) = window.set_position(LogicalPosition::new(x, y)) {
                    eprintln!("❌ Failed to set notification window position: {}", e);
                }

                // Debug: Print actual window size after creation
                if let Ok(size) = window.outer_size() {
                    println!("📍 Actual window outer size: {:?}", size);
                }
                if let Ok(size) = window.inner_size() {
                    println!("📍 Actual window inner size: {:?}", size);
                }
                if let Ok(pos) = window.outer_position() {
                    println!("📍 Actual window position: {:?}", pos);
                }
            }

            // Show the window
            if let Err(e) = window.show() {
                eprintln!("❌ Failed to show notification window: {}", e);
            } else {
                println!("✅ Notification window created and shown");
            }

            // Close window after 5 seconds
            let window_clone = window.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(5));
                let _ = window_clone.close();
            });
        }
        Err(e) => {
            eprintln!("❌ Failed to create notification window: {}", e);
        }
    }
}
