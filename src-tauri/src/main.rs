#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod attachment_limits;
mod commands;
mod db;
mod idle_manager;
mod models;
mod oauth2_config;

use commands::{
    complete_oauth2_flow, delete_account, delete_email, download_attachment, fetch_email_body,
    fetch_email_body_cached, fetch_emails, fetch_folders, forward_email, get_attachment_size_limit,
    get_last_sync_time, get_notification_enabled, get_sound_enabled, get_sync_interval,
    listen_for_oauth_callback, load_account_configs, load_attachments_info, load_emails_from_cache,
    load_folders, move_email_to_trash, reply_email, save_account_config, save_attachment_to_file,
    send_email, set_notification_enabled, set_sound_enabled, set_sync_interval, should_sync,
    start_oauth2_flow, sync_emails, sync_folders,
};
use idle_manager::{IdleCommand, IdleManager};
use models::AccountConfig;
use std::sync::{Arc, Mutex};
use tauri::{command, Manager, State};

// IDLE manager commands
#[command]
async fn start_idle(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    account_id: i32,
    folder_name: String,
    config: AccountConfig,
) -> Result<(), String> {
    let manager = idle_manager.lock().unwrap();
    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::Start {
            account_id,
            folder_name,
            config,
        })?;
    }
    Ok(())
}

#[command]
async fn stop_idle(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    account_id: i32,
    folder_name: String,
) -> Result<(), String> {
    let manager = idle_manager.lock().unwrap();
    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::Stop {
            account_id,
            folder_name,
        })?;
    }
    Ok(())
}

#[command]
async fn stop_all_idle(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
) -> Result<(), String> {
    let manager = idle_manager.lock().unwrap();
    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::StopAll)?;
    }
    Ok(())
}

#[command]
fn is_idle_active(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    account_id: i32,
    folder_name: String,
) -> Result<bool, String> {
    let manager = idle_manager.lock().unwrap();
    if let Some(ref mgr) = *manager {
        Ok(mgr.is_active(account_id, &folder_name))
    } else {
        Ok(false)
    }
}

#[command]
async fn start_idle_for_account(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    config: AccountConfig,
) -> Result<(), String> {
    let manager = idle_manager.lock().unwrap();
    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::StartAllForAccount { config })?;
    }
    Ok(())
}

#[command]
async fn stop_idle_for_account(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
    account_id: i32,
) -> Result<(), String> {
    let manager = idle_manager.lock().unwrap();
    if let Some(ref mgr) = *manager {
        mgr.send_command(IdleCommand::StopAllForAccount { account_id })?;
    }
    Ok(())
}

#[command]
async fn start_idle_for_all_accounts(
    idle_manager: State<'_, Arc<Mutex<Option<IdleManager>>>>,
) -> Result<(), String> {
    println!("🚀 Starting IDLE monitoring for all accounts...");

    // Load all accounts from database
    let accounts = load_account_configs().await?;

    println!("  📧 Found {} accounts", accounts.len());

    let manager = idle_manager.lock().unwrap();
    if let Some(ref mgr) = *manager {
        for account in accounts {
            println!("  🔄 Starting IDLE for account: {}", account.email);
            mgr.send_command(IdleCommand::StartAllForAccount {
                config: account.clone(),
            })?;
        }
    }

    println!("✅ IDLE monitoring started for all accounts");
    Ok(())
}

#[tokio::main]
async fn main() {
    db::init().await.expect("Failed to initialize database");

    match load_account_configs().await {
        Ok(accounts) => println!(
            "🚀 App startup: Loaded {} accounts from database.",
            accounts.len()
        ),
        Err(e) => eprintln!("Error loading accounts on startup: {}", e),
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Initialize IDLE manager
            let idle_manager = Arc::new(Mutex::new(Some(IdleManager::new(app.handle().clone()))));
            app.manage(idle_manager.clone());

            println!("✅ IDLE manager initialized");

            // Auto-start IDLE monitoring for all accounts on app startup
            let idle_manager_clone = idle_manager.clone();
            tokio::spawn(async move {
                // Wait a bit for the app to fully initialize
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                println!("🚀 Auto-starting IDLE monitoring for all accounts...");

                match load_account_configs().await {
                    Ok(accounts) => {
                        println!("  📧 Found {} accounts to monitor", accounts.len());

                        let manager = idle_manager_clone.lock().unwrap();
                        if let Some(ref mgr) = *manager {
                            for account in accounts {
                                println!("  🔄 Starting IDLE for account: {}", account.email);
                                if let Err(e) = mgr.send_command(IdleCommand::StartAllForAccount {
                                    config: account.clone(),
                                }) {
                                    eprintln!(
                                        "❌ Failed to start IDLE for account {}: {}",
                                        account.email, e
                                    );
                                }
                            }
                            println!("✅ IDLE auto-start completed");
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to load accounts for IDLE auto-start: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_account_config,
            load_account_configs,
            delete_account,
            fetch_emails,
            fetch_email_body,
            fetch_email_body_cached,
            load_emails_from_cache,
            sync_emails,
            get_last_sync_time,
            should_sync,
            get_sync_interval,
            set_sync_interval,
            get_notification_enabled,
            set_notification_enabled,
            get_sound_enabled,
            set_sound_enabled,
            move_email_to_trash,
            delete_email,
            send_email,
            reply_email,
            forward_email,
            get_attachment_size_limit,
            fetch_folders,
            sync_folders,
            load_folders,
            start_oauth2_flow,
            listen_for_oauth_callback,
            complete_oauth2_flow,
            load_attachments_info,
            download_attachment,
            save_attachment_to_file,
            start_idle,
            stop_idle,
            stop_all_idle,
            is_idle_active,
            start_idle_for_account,
            stop_idle_for_account,
            start_idle_for_all_accounts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
