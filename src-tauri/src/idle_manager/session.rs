// IDLE session handling
// This module manages individual IMAP IDLE sessions and connection loops

use super::notification::send_notification;
use super::types::{IdleEvent, IdleEventType};
use crate::commands::emails::imap_helpers;
use crate::commands::utils::ensure_valid_token;
use crate::models::AccountConfig;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// IDLE connection loop for a single folder
pub async fn idle_connection_loop(
    app_handle: AppHandle,
    account_id: i32,
    folder_name: String,
    config: AccountConfig,
) {
    loop {
        tracing::info!(
            account_id = account_id,
            folder = %folder_name,
            "Establishing IDLE connection"
        );

        match idle_session(&app_handle, account_id, &folder_name, &config).await {
            Ok(_) => {
                tracing::info!(
                    account_id = account_id,
                    folder = %folder_name,
                    "IDLE session ended normally"
                );
            }
            Err(e) => {
                tracing::error!(
                    account_id = account_id,
                    folder = %folder_name,
                    error = %e,
                    "IDLE session error"
                );

                // Check if error is due to IDLE not being supported
                if e.contains("does not support IDLE") {
                    tracing::warn!("IDLE not supported by server, stopping IDLE monitoring for this account/folder");
                    tracing::info!(
                        "Tip: Use manual sync (Sync Mail button) to check for new emails"
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
        tracing::info!("Waiting 30 seconds before reconnecting...");
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

        tracing::info!("IDLE IMAP authentication successful");

        // Check if server supports IDLE capability
        let capabilities = imap_session
            .capabilities()
            .map_err(|e| format!("Failed to get capabilities: {}", e))?;

        // Check if IDLE is in the capabilities list
        let has_idle = capabilities.has_str("IDLE");

        if !has_idle {
            tracing::warn!("Server does not support IDLE capability (RFC 2177)");
            tracing::info!("Use manual sync (Sync Mail button) to check for new emails");
            return Err("Server does not support IDLE extension (RFC 2177)".to_string());
        }

        tracing::info!("Server supports IDLE capability");

        // SELECT folder
        let mailbox = imap_session
            .select(&folder_name_owned)
            .map_err(|e| format!("Cannot select folder: {}", e))?;

        tracing::info!(
            folder = %folder_name_owned,
            "IDLE mode activated"
        );
        tracing::info!(
            exists = mailbox.exists,
            recent = mailbox.recent,
            "Initial mailbox state"
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

        tracing::debug!("IDLE waiting for changes...");

        // Wait for mailbox changes using the new wait_while API
        let wait_result = idle_handle.wait_while(|response: UnsolicitedResponse| {
            match response {
                UnsolicitedResponse::Exists(count) => {
                    tracing::debug!(count = count, "IDLE: EXISTS");

                    // Detect new messages
                    if count > prev_exists {
                        let new_count = count - prev_exists;
                        tracing::info!(count = new_count, "Detected new message(s)");

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

                    // Continue waiting for more changes instead of stopping IDLE
                    true
                }
                UnsolicitedResponse::Recent(count) => {
                    tracing::debug!(count = count, "IDLE: RECENT");
                    // Continue waiting
                    true
                }
                UnsolicitedResponse::Expunge(seq) => {
                    tracing::info!(seq = seq, "IDLE: EXPUNGE");

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
                    tracing::debug!(id = id, "IDLE: FETCH");

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
                    tracing::debug!(response = ?response, "IDLE: Other response");
                    // Continue waiting for other responses
                    true
                }
            }
        });

        match wait_result {
            Ok(_outcome) => {
                tracing::info!("IDLE session completed successfully");
                Ok(())
            }
            Err(e) => {
                tracing::error!(error = %e, "IDLE wait error");
                Err(format!("IDLE error: {}", e))
            }
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(())
}
