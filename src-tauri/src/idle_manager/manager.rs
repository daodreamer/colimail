// IDLE manager core
// This module manages IDLE connections and command processing

use super::session::idle_connection_loop;
use super::types::IdleCommand;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tokio::sync::mpsc;

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

                    tracing::info!(
                        account_id = account_id,
                        folder = %folder_name,
                        "Starting IDLE connection"
                    );

                    // Spawn new IDLE task
                    let app_handle_clone = app_handle.clone();
                    let active_connections_clone = active_connections.clone();

                    let task = tokio::spawn(async move {
                        idle_connection_loop(
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
                        tracing::info!(
                            account_id = account_id,
                            folder = %folder_name,
                            "Stopping IDLE connection"
                        );
                        task.abort();
                        active_connections.lock().unwrap().remove(&key);
                    }
                }

                IdleCommand::StopAll => {
                    tracing::info!("Stopping all IDLE connections");

                    for (_, task) in tasks.drain() {
                        task.abort();
                    }

                    active_connections.lock().unwrap().clear();
                }

                IdleCommand::StartAllForAccount { config } => {
                    let account_id = match config.id {
                        Some(id) => id,
                        None => {
                            tracing::error!("Cannot start IDLE: account ID missing");
                            continue;
                        }
                    };

                    tracing::info!(
                        account_id = account_id,
                        email = %config.email,
                        "Starting IDLE for INBOX"
                    );

                    // Only start IDLE for INBOX to avoid hitting connection limits
                    // Most email providers (Gmail, Outlook, etc.) limit concurrent IMAP connections to 10-15
                    // IDLE monitoring is primarily needed for INBOX to get new email notifications
                    let inbox_folders = ["INBOX", "收件箱"];

                    // Load folders to find the actual INBOX folder name
                    match crate::commands::load_folders(account_id).await {
                        Ok(folders) => {
                            // Find INBOX folder (case-insensitive match)
                            let inbox_folder = folders.iter().find(|f| {
                                let folder_lower = f.name.to_lowercase();
                                inbox_folders
                                    .iter()
                                    .any(|inbox| folder_lower.contains(&inbox.to_lowercase()))
                            });

                            if let Some(folder) = inbox_folder {
                                let key = (account_id, folder.name.clone());

                                // Stop existing connection if any
                                if let Some(task) = tasks.remove(&key) {
                                    task.abort();
                                }

                                // Mark as active
                                active_connections.lock().unwrap().insert(key.clone(), ());

                                tracing::info!(
                                    folder = %folder.display_name,
                                    "Starting IDLE for folder"
                                );

                                // Clone necessary data
                                let app_handle_clone = app_handle.clone();
                                let active_connections_clone = active_connections.clone();
                                let config_clone = config.clone();
                                let folder_name = folder.name.clone();

                                // Spawn IDLE task
                                let task = tokio::spawn(async move {
                                    idle_connection_loop(
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
                                tracing::info!("Started IDLE monitoring for INBOX");
                            } else {
                                tracing::error!(
                                    account_id = account_id,
                                    "Could not find INBOX folder"
                                );
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                account_id = account_id,
                                error = %e,
                                "Failed to load folders"
                            );
                        }
                    }
                }

                IdleCommand::StopAllForAccount { account_id } => {
                    tracing::info!(
                        account_id = account_id,
                        "Stopping all IDLE connections for account"
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
                            tracing::info!(
                                folder = %key.1,
                                "Stopped IDLE for folder"
                            );
                        }
                    }

                    tracing::info!(
                        account_id = account_id,
                        "Stopped all IDLE connections for account"
                    );
                }
            }
        }
    }

    /// Check if a connection is active
    pub fn is_active(&self, account_id: i32, folder_name: &str) -> bool {
        self.active_connections
            .lock()
            .unwrap()
            .contains_key(&(account_id, folder_name.to_string()))
    }
}
