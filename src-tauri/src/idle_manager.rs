use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType};
use native_tls::TlsConnector;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

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

/// OAuth2 authenticator for IMAP
struct OAuth2 {
    user: String,
    access_token: String,
}

impl imap::Authenticator for OAuth2 {
    type Response = String;
    fn process(&self, _: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
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
        // Ensure we have a valid access token (refresh if needed)
        let config_refreshed = ensure_valid_token(config.clone()).await?;

        let config_clone = config_refreshed.clone();
        let folder_name_owned = folder_name.to_string();
        let app_handle_clone = app_handle.clone();

        tokio::task::spawn_blocking(move || {
            let domain = config_clone.imap_server.as_str();
            let port = config_clone.imap_port;
            let email = config_clone.email.as_str();

            let tls = TlsConnector::builder().build().map_err(|e| e.to_string())?;
            let client = imap::connect((domain, port), domain, &tls).map_err(|e| e.to_string())?;

            // Authenticate
            let mut imap_session = match config_clone.auth_type {
                Some(AuthType::OAuth2) => {
                    let access_token = config_clone
                        .access_token
                        .as_ref()
                        .ok_or("Access token is required for OAuth2 authentication")?;

                    let oauth2 = OAuth2 {
                        user: email.to_string(),
                        access_token: access_token.clone(),
                    };

                    client
                        .authenticate("XOAUTH2", &oauth2)
                        .map_err(|e| format!("OAuth2 authentication failed: {}", e.0))?
                }
                _ => {
                    let password = config_clone
                        .password
                        .as_ref()
                        .ok_or("Password is required for basic authentication")?;

                    client.login(email, password).map_err(|e| e.0.to_string())?
                }
            };

            println!("✅ IDLE IMAP authentication successful");

            // SELECT folder
            let _mailbox = imap_session
                .select(&folder_name_owned)
                .map_err(|e| format!("Cannot select folder: {}", e))?;

            println!("📥 IDLE mode activated for {}", folder_name_owned);

            // IDLE loop with 15-minute timeout (recommended by RFC)
            let idle_duration = Duration::from_secs(15 * 60);
            let start = std::time::Instant::now();

            loop {
                // Check if we should refresh IDLE (every 15 minutes)
                if start.elapsed() >= idle_duration {
                    println!("🔄 Refreshing IDLE connection (15-minute timeout)");
                    break;
                }

                // Enter IDLE mode
                let idle_handle = imap_session
                    .idle()
                    .map_err(|e| format!("Failed to enter IDLE: {}", e))?;

                // Wait for notifications with 5-minute timeout
                let idle_result = idle_handle.wait_keepalive();

                match idle_result {
                    Ok(()) => {
                        println!("📬 Received IDLE notification");

                        // Exit IDLE to process changes
                        // The idle handle will automatically exit when dropped

                        // Emit event to frontend to trigger sync
                        let _ = app_handle_clone.emit(
                            "idle-event",
                            IdleEvent {
                                account_id,
                                folder_name: folder_name_owned.clone(),
                                event_type: IdleEventType::NewMessages { count: 1 },
                            },
                        );

                        println!("✨ Emitted IDLE event to frontend");
                    }
                    Err(e) => {
                        eprintln!("⚠️ IDLE wait error: {}", e);
                        return Err(format!("IDLE wait error: {}", e));
                    }
                }

                // Small delay before re-entering IDLE
                std::thread::sleep(Duration::from_millis(100));
            }

            let _ = imap_session.logout();
            Ok(())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Check if a connection is active
    pub fn is_active(&self, account_id: i32, folder_name: &str) -> bool {
        self.active_connections
            .lock()
            .unwrap()
            .contains_key(&(account_id, folder_name.to_string()))
    }
}
