use crate::commands::emails::imap_helpers;
use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::AccountConfig;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, LogicalPosition, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::mpsc;

/// Notification data to be queued
#[derive(Debug, Clone)]
struct NotificationData {
    title: String,
    from: String,
    subject: String,
}

/// Global notification queue
static NOTIFICATION_QUEUE: Lazy<Arc<Mutex<Vec<NotificationData>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

/// Flag to track if notification worker is running
static NOTIFICATION_WORKER_RUNNING: Lazy<Arc<Mutex<bool>>> =
    Lazy::new(|| Arc::new(Mutex::new(false)));

/// Command to control the IDLE manager
#[derive(Debug)]
pub enum IdleCommand {
    Start {
        account_id: i32,
        folder_name: String,
        config: AccountConfig,
    },
    Stop {
        account_id: i32,
        folder_name: String,
    },
    StopAll,
    StartAllForAccount {
        config: AccountConfig,
    },
    StopAllForAccount {
        account_id: i32,
    },
}

/// Event emitted by IDLE connections
#[derive(Debug, Clone, serde::Serialize)]
pub struct IdleEvent {
    pub account_id: i32,
    pub folder_name: String,
    pub event_type: IdleEventType,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum IdleEventType {
    NewMessages { count: u32 },
    Expunge { uid: u32 },
    FlagsChanged { uid: u32 },
    ConnectionLost,
}

/// Global IDLE manager instance
pub struct IdleManager {
    command_tx: mpsc::UnboundedSender<IdleCommand>,
    active_connections: Arc<Mutex<HashMap<(i32, String), ()>>>,
}

impl IdleManager {
    /// Create a new IDLE manager
    pub fn new(app_handle: AppHandle) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let active_connections = Arc::new(Mutex::new(HashMap::new()));

        let active_connections_clone = active_connections.clone();

        // Spawn the manager task
        tokio::spawn(async move {
            Self::run_manager(app_handle, command_rx, active_connections_clone).await;
        });

        IdleManager {
            command_tx,
            active_connections,
        }
    }

    /// Send a command to the manager
    pub fn send_command(&self, cmd: IdleCommand) -> Result<(), String> {
        self.command_tx
            .send(cmd)
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// Main manager loop
    async fn run_manager(
        app_handle: AppHandle,
        mut command_rx: mpsc::UnboundedReceiver<IdleCommand>,
        active_connections: Arc<Mutex<HashMap<(i32, String), ()>>>,
    ) {
        let mut tasks: HashMap<(i32, String), tokio::task::JoinHandle<()>> = HashMap::new();

        while let Some(cmd) = command_rx.recv().await {
            match cmd {
                IdleCommand::Start {
                    account_id,
                    folder_name,
                    config,
                } => {
                    let key = (account_id, folder_name.clone());

                    // Stop existing connection if any
                    if let Some(task) = tasks.remove(&key) {
                        task.abort();
                    }

                    // Mark as active
                    active_connections.lock().unwrap().insert(key.clone(), ());

                    println!(
                        "🔄 Starting IDLE connection for account {} folder {}",
                        account_id, folder_name
                    );

                    // Spawn new IDLE task
                    let app_handle_clone = app_handle.clone();
                    let active_connections_clone = active_connections.clone();

                    let task = tokio::spawn(async move {
                        Self::idle_connection_loop(
                            app_handle_clone,
                            account_id,
                            folder_name.clone(),
                            config,
                        )
                        .await;

                        // Remove from active connections when done
                        active_connections_clone
                            .lock()
                            .unwrap()
                            .remove(&(account_id, folder_name));
                    });

                    tasks.insert(key, task);
                }

                IdleCommand::Stop {
                    account_id,
                    folder_name,
                } => {
                    let key = (account_id, folder_name.clone());

                    if let Some(task) = tasks.remove(&key) {
                        println!(
                            "⏹️ Stopping IDLE connection for account {} folder {}",
                            account_id, folder_name
                        );
                        task.abort();
                        active_connections.lock().unwrap().remove(&key);
                    }
                }

                IdleCommand::StopAll => {
                    println!("⏹️ Stopping all IDLE connections");

                    for (_, task) in tasks.drain() {
                        task.abort();
                    }

                    active_connections.lock().unwrap().clear();
                }

                IdleCommand::StartAllForAccount { config } => {
                    let account_id = match config.id {
                        Some(id) => id,
                        None => {
                            eprintln!("❌ Cannot start IDLE: account ID missing");
                            continue;
                        }
                    };

                    println!(
                        "🚀 Starting IDLE for all folders of account {} ({})",
                        account_id, config.email
                    );

                    // Load folders from database
                    match crate::commands::load_folders(account_id).await {
                        Ok(folders) => {
                            println!("  📁 Found {} folders to monitor", folders.len());

                            for folder in &folders {
                                let key = (account_id, folder.name.clone());

                                // Stop existing connection if any
                                if let Some(task) = tasks.remove(&key) {
                                    task.abort();
                                }

                                // Mark as active
                                active_connections.lock().unwrap().insert(key.clone(), ());

                                println!("  🔄 Starting IDLE for folder: {}", folder.display_name);

                                // Clone necessary data
                                let app_handle_clone = app_handle.clone();
                                let active_connections_clone = active_connections.clone();
                                let config_clone = config.clone();
                                let folder_name = folder.name.clone();

                                // Spawn IDLE task
                                let task = tokio::spawn(async move {
                                    Self::idle_connection_loop(
                                        app_handle_clone,
                                        account_id,
                                        folder_name.clone(),
                                        config_clone,
                                    )
                                    .await;

                                    // Remove from active connections when done
                                    active_connections_clone
                                        .lock()
                                        .unwrap()
                                        .remove(&(account_id, folder_name));
                                });

                                tasks.insert(key, task);
                            }

                            println!("✅ Started IDLE monitoring for {} folders", folders.len());
                        }
                        Err(e) => {
                            eprintln!(
                                "❌ Failed to load folders for account {}: {}",
                                account_id, e
                            );
                        }
                    }
                }

                IdleCommand::StopAllForAccount { account_id } => {
                    println!(
                        "⏹️ Stopping all IDLE connections for account {}",
                        account_id
                    );

                    // Find and stop all tasks for this account
                    let keys_to_remove: Vec<_> = tasks
                        .keys()
                        .filter(|(acc_id, _)| *acc_id == account_id)
                        .cloned()
                        .collect();

                    for key in keys_to_remove {
                        if let Some(task) = tasks.remove(&key) {
                            task.abort();
                            active_connections.lock().unwrap().remove(&key);
                            println!("  ⏹️ Stopped IDLE for folder: {}", key.1);
                        }
                    }

                    println!("✅ Stopped all IDLE connections for account {}", account_id);
                }
            }
        }
    }

    /// IDLE connection loop for a single folder
    async fn idle_connection_loop(
        app_handle: AppHandle,
        account_id: i32,
        folder_name: String,
        config: AccountConfig,
    ) {
        loop {
            println!(
                "🔌 Establishing IDLE connection for account {} folder {}",
                account_id, folder_name
            );

            match Self::idle_session(&app_handle, account_id, &folder_name, &config).await {
                Ok(_) => {
                    println!(
                        "✅ IDLE session ended normally for account {} folder {}",
                        account_id, folder_name
                    );
                }
                Err(e) => {
                    eprintln!(
                        "❌ IDLE session error for account {} folder {}: {}",
                        account_id, folder_name, e
                    );

                    // Check if error is due to IDLE not being supported
                    if e.contains("does not support IDLE") {
                        println!("⚠️ IDLE not supported by server, stopping IDLE monitoring for this account/folder");
                        println!(
                            "💡 Tip: Use manual sync (Sync Mail button) to check for new emails"
                        );
                        // Stop the IDLE loop for this account/folder
                        break;
                    }

                    // Emit connection lost event
                    let _ = app_handle.emit(
                        "idle-event",
                        IdleEvent {
                            account_id,
                            folder_name: folder_name.clone(),
                            event_type: IdleEventType::ConnectionLost,
                        },
                    );
                }
            }

            // Wait before reconnecting (exponential backoff would be better)
            println!("⏳ Waiting 30 seconds before reconnecting...");
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }

    /// Run a single IDLE session
    async fn idle_session(
        app_handle: &AppHandle,
        account_id: i32,
        folder_name: &str,
        config: &AccountConfig,
    ) -> Result<(), String> {
        use imap::types::UnsolicitedResponse;

        // Ensure we have a valid access token (refresh if needed)
        let config_refreshed = ensure_valid_token(config.clone()).await?;

        let config_clone = config_refreshed.clone();
        let folder_name_owned = folder_name.to_string();
        let app_handle_clone = app_handle.clone();

        tokio::task::spawn_blocking(move || {
            // Use helper function for connection with imap 3.0.0 API
            let mut imap_session = imap_helpers::connect_and_login(&config_clone)?;

            println!("✅ IDLE IMAP authentication successful");

            // Check if server supports IDLE capability
            let capabilities = imap_session
                .capabilities()
                .map_err(|e| format!("Failed to get capabilities: {}", e))?;

            // Check if IDLE is in the capabilities list
            let has_idle = capabilities.has_str("IDLE");

            if !has_idle {
                println!("⚠️ Server does not support IDLE capability (RFC 2177)");
                println!("💡 Use manual sync (Sync Mail button) to check for new emails");
                return Err("Server does not support IDLE extension (RFC 2177)".to_string());
            }

            println!("✅ Server supports IDLE capability");

            // SELECT folder
            let mailbox = imap_session
                .select(&folder_name_owned)
                .map_err(|e| format!("Cannot select folder: {}", e))?;

            println!("📥 IDLE mode activated for {}", folder_name_owned);
            println!(
                "📊 Initial mailbox state - EXISTS: {}, RECENT: {}",
                mailbox.exists, mailbox.recent
            );

            // Track initial state
            let mut prev_exists = mailbox.exists;

            // Start IDLE session with imap 3.0.0 API
            // Note: .idle() returns Handle directly, not Result
            let mut idle_handle = imap_session.idle();

            // Set keepalive to true (default, but explicit for clarity)
            idle_handle.keepalive(true);

            // Set timeout to 29 minutes (default, per RFC 2177)
            idle_handle.timeout(Duration::from_secs(29 * 60));

            println!("⏳ IDLE waiting for changes...");

            // Wait for mailbox changes using the new wait_while API
            let wait_result = idle_handle.wait_while(|response: UnsolicitedResponse| {
                match response {
                    UnsolicitedResponse::Exists(count) => {
                        println!("📨 IDLE: EXISTS = {}", count);

                        // Detect new messages
                        if count > prev_exists {
                            let new_count = count - prev_exists;
                            println!("✨ Detected {} new message(s)", new_count);

                            // Emit event to frontend
                            let _ = app_handle_clone.emit(
                                "idle-event",
                                IdleEvent {
                                    account_id,
                                    folder_name: folder_name_owned.clone(),
                                    event_type: IdleEventType::NewMessages { count: new_count },
                                },
                            );

                            // Send desktop notification
                            let app_handle_clone2 = app_handle_clone.clone();
                            let folder_name_clone = folder_name_owned.clone();
                            tokio::spawn(async move {
                                send_notification(
                                    &app_handle_clone2,
                                    account_id,
                                    &folder_name_clone,
                                    new_count,
                                )
                                .await;
                            });
                        }

                        prev_exists = count;

                        // Return false to stop IDLE after detecting change
                        false
                    }
                    UnsolicitedResponse::Recent(count) => {
                        println!("📬 IDLE: RECENT = {}", count);
                        // Continue waiting
                        true
                    }
                    UnsolicitedResponse::Expunge(seq) => {
                        println!("🗑️ IDLE: EXPUNGE seq={}", seq);

                        // Emit expunge event
                        let _ = app_handle_clone.emit(
                            "idle-event",
                            IdleEvent {
                                account_id,
                                folder_name: folder_name_owned.clone(),
                                event_type: IdleEventType::Expunge { uid: seq },
                            },
                        );

                        // Continue waiting
                        true
                    }
                    UnsolicitedResponse::Fetch { id, .. } => {
                        println!("🏴 IDLE: FETCH id={}", id);

                        // Emit flags changed event
                        let _ = app_handle_clone.emit(
                            "idle-event",
                            IdleEvent {
                                account_id,
                                folder_name: folder_name_owned.clone(),
                                event_type: IdleEventType::FlagsChanged { uid: id },
                            },
                        );

                        // Continue waiting
                        true
                    }
                    _ => {
                        println!("📡 IDLE: Other response: {:?}", response);
                        // Continue waiting for other responses
                        true
                    }
                }
            });

            match wait_result {
                Ok(_outcome) => {
                    println!("✅ IDLE session completed successfully");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ IDLE wait error: {}", e);
                    Err(format!("IDLE error: {}", e))
                }
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))??;

        Ok(())
    }

    /// Check if a connection is active
    pub fn is_active(&self, account_id: i32, folder_name: &str) -> bool {
        self.active_connections
            .lock()
            .unwrap()
            .contains_key(&(account_id, folder_name.to_string()))
    }
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
async fn send_notification(app_handle: &AppHandle, account_id: i32, folder_name: &str, count: u32) {
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
