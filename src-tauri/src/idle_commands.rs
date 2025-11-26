use crate::commands::load_account_configs;
use crate::idle_manager::{IdleCommand, IdleManager};
use crate::models::AccountConfig;
use std::sync::{Arc, Mutex};
use tauri::{command, State};

/// Start IDLE monitoring for a specific account and folder
#[command]
pub async fn start_idle(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    account_id: i32,
    folder_name: String,
    config: AccountConfig,
) -> Result<(), String> {
    let manager = idle_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::Start {
            account_id,
            folder_name,
            config,
        })?;
    } else {
        tracing::warn!("IDLE manager not initialized");
    }
    Ok(())
}

/// Stop IDLE monitoring for a specific account and folder
#[command]
pub async fn stop_idle(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    account_id: i32,
    folder_name: String,
) -> Result<(), String> {
    let manager = idle_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::Stop {
            account_id,
            folder_name,
        })?;
    } else {
        tracing::warn!("IDLE manager not initialized");
    }
    Ok(())
}

/// Stop all IDLE monitoring sessions
#[command]
pub async fn stop_all_idle(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
) -> Result<(), String> {
    let manager = idle_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::StopAll)?;
    } else {
        tracing::warn!("IDLE manager not initialized");
    }
    Ok(())
}

/// Check if IDLE monitoring is active for a specific account and folder
#[command]
pub fn is_idle_active(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    account_id: i32,
    folder_name: String,
) -> Result<bool, String> {
    let manager = idle_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(ref mgr) = *manager {
        Ok(mgr.is_active(account_id, &folder_name))
    } else {
        Ok(false)
    }
}

/// Start IDLE monitoring for all folders of a specific account
#[command]
pub async fn start_idle_for_account(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    config: AccountConfig,
) -> Result<(), String> {
    let manager = idle_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::StartAllForAccount { config })?;
    } else {
        tracing::warn!("IDLE manager not initialized");
    }
    Ok(())
}

/// Stop IDLE monitoring for all folders of a specific account
#[command]
pub async fn stop_idle_for_account(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    account_id: i32,
) -> Result<(), String> {
    let manager = idle_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::StopAllForAccount { account_id })?;
    } else {
        tracing::warn!("IDLE manager not initialized");
    }
    Ok(())
}

/// Start IDLE monitoring for all accounts
#[command]
pub async fn start_idle_for_all_accounts(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
) -> Result<(), String> {
    tracing::info!("Starting IDLE monitoring for all accounts");

    // Load all accounts from database
    let accounts = load_account_configs().await?;

    tracing::info!(account_count = accounts.len(), "Found accounts to monitor");

    let manager = idle_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(ref mgr) = *manager {
        for account in accounts {
            tracing::info!(email = %account.email, "Starting IDLE for account");
            mgr.send_command(IdleCommand::StartAllForAccount {
                config: account.clone(),
            })?;
        }
    } else {
        tracing::warn!("IDLE manager not initialized");
    }

    tracing::info!("IDLE monitoring started for all accounts");
    Ok(())
}
