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
    get_last_sync_time, get_sync_interval, listen_for_oauth_callback, load_account_configs,
    load_attachments_info, load_emails_from_cache, load_folders, move_email_to_trash, reply_email,
    save_account_config, save_attachment_to_file, send_email, set_sync_interval, should_sync,
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
        .setup(|app| {
            // Initialize IDLE manager
            let idle_manager = Arc::new(Mutex::new(Some(IdleManager::new(app.handle().clone()))));
            app.manage(idle_manager);

            println!("✅ IDLE manager initialized");
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
            is_idle_active
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
