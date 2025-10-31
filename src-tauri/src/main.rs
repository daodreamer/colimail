#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod attachment_limits;
mod commands;
mod db;
mod idle_manager;
mod logger;
mod models;
mod oauth2_config;
mod security;

use commands::{
    check_folder_capabilities, complete_oauth2_flow, create_local_folder, create_remote_folder,
    delete_account, delete_app_user, delete_draft, delete_email, delete_local_folder,
    delete_remote_folder, delete_secure_storage, detect_display_name_from_sent,
    download_attachment, fetch_email_body, fetch_email_body_cached, fetch_emails, fetch_folders,
    forward_email, get_app_user, get_attachment_size_limit, get_current_log_file,
    get_last_sync_time, get_log_directory, get_minimize_to_tray, get_notification_enabled,
    get_secure_storage, get_sound_enabled, get_sync_interval, list_drafts, list_log_files,
    listen_for_oauth_callback, load_account_configs, load_attachments_info, load_draft,
    load_emails_from_cache, load_folders, mark_email_as_flagged, mark_email_as_read,
    mark_email_as_unflagged, mark_email_as_unread, move_email_to_trash, read_log_file,
    read_recent_logs, reply_email, save_account_config, save_attachment_to_file, save_draft,
    send_email, set_minimize_to_tray, set_notification_enabled, set_secure_storage,
    set_sound_enabled, set_sync_interval, should_sync, start_oauth2_flow, sync_app_user,
    sync_email_flags, sync_emails, sync_folders, sync_specific_email_flags, test_connection,
};
use idle_manager::{IdleCommand, IdleManager};
use models::AccountConfig;
use std::sync::{Arc, Mutex};
use tauri::{
    command,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State,
};

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
    tracing::info!("Starting IDLE monitoring for all accounts");

    // Load all accounts from database
    let accounts = load_account_configs().await?;

    tracing::info!(account_count = accounts.len(), "Found accounts to monitor");

    let manager = idle_manager.lock().unwrap();
    if let Some(ref mgr) = *manager {
        for account in accounts {
            tracing::info!(email = %account.email, "Starting IDLE for account");
            mgr.send_command(IdleCommand::StartAllForAccount {
                config: account.clone(),
            })?;
        }
    }

    tracing::info!("IDLE monitoring started for all accounts");
    Ok(())
}

#[tokio::main]
async fn main() {
    // Initialize logging system first
    logger::init().expect("Failed to initialize logging system");

    tracing::info!("Starting Colimail application");

    db::init().await.expect("Failed to initialize database");

    match load_account_configs().await {
        Ok(accounts) => {
            tracing::info!(
                account_count = accounts.len(),
                "App startup: Loaded accounts from database"
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "Error loading accounts on startup");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            oauth2_config::init_credentials(app.handle());

            // TODO: Setup deep link handler for OAuth callbacks
            // The deep-link plugin API needs to be configured properly
            // For now, users can manually copy the callback URL

            // Request notification permission (important for Windows)
            use tauri_plugin_notification::NotificationExt;
            let notification = app.handle().notification();

            // Check and request permission
            match notification.permission_state() {
                Ok(state) => {
                    tracing::info!(?state, "Notification permission state");
                    if state.to_string() != "granted" {
                        tracing::info!("Requesting notification permission");
                        match notification.request_permission() {
                            Ok(_) => tracing::info!("Notification permission requested"),
                            Err(e) => {
                                tracing::error!(error = %e, "Failed to request notification permission");
                            }
                        }
                    }
                }
                Err(e) => tracing::error!(error = %e, "Failed to check notification permission"),
            }

            // Setup system tray
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            // Emit event to open settings dialog
                            let _ = window.emit("open-settings", ());
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // Left click: toggle window visibility
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Handle window close event - minimize to tray or close based on user setting
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let app_handle_clone = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            // Check user preference for minimize to tray
                            let minimize_to_tray = get_minimize_to_tray().await.unwrap_or(true);

                            if minimize_to_tray {
                                // Hide window instead of closing
                                if let Some(window) = app_handle_clone.get_webview_window("main") {
                                    let _ = window.hide();
                                }
                            } else {
                                // Exit the application
                                app_handle_clone.exit(0);
                            }
                        });

                        // Prevent default close behavior
                        api.prevent_close();
                    }
                });
            }

            // Initialize IDLE manager
            let idle_manager = Arc::new(Mutex::new(Some(IdleManager::new(app.handle().clone()))));
            app.manage(idle_manager.clone());

            tracing::info!("IDLE manager initialized");

            // Auto-start IDLE monitoring for all accounts on app startup
            let idle_manager_clone = idle_manager.clone();
            tokio::spawn(async move {
                // Wait a bit for the app to fully initialize
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                tracing::info!("Auto-starting IDLE monitoring for all accounts");

                match load_account_configs().await {
                    Ok(accounts) => {
                        tracing::info!(account_count = accounts.len(), "Found accounts to monitor");

                        let manager = idle_manager_clone.lock().unwrap();
                        if let Some(ref mgr) = *manager {
                            for account in accounts {
                                tracing::info!(email = %account.email, "Starting IDLE for account");
                                if let Err(e) = mgr.send_command(IdleCommand::StartAllForAccount {
                                    config: account.clone(),
                                }) {
                                    tracing::error!(
                                        email = %account.email,
                                        error = %e,
                                        "Failed to start IDLE for account"
                                    );
                                }
                            }
                            tracing::info!("IDLE auto-start completed");
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to load accounts for IDLE auto-start");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_account_config,
            load_account_configs,
            delete_account,
            detect_display_name_from_sent,
            fetch_emails,
            fetch_email_body,
            fetch_email_body_cached,
            load_emails_from_cache,
            sync_emails,
            sync_email_flags,
            sync_specific_email_flags,
            get_last_sync_time,
            should_sync,
            get_sync_interval,
            set_sync_interval,
            get_notification_enabled,
            set_notification_enabled,
            get_sound_enabled,
            set_sound_enabled,
            get_minimize_to_tray,
            set_minimize_to_tray,
            move_email_to_trash,
            delete_email,
            send_email,
            reply_email,
            forward_email,
            get_attachment_size_limit,
            fetch_folders,
            sync_folders,
            load_folders,
            check_folder_capabilities,
            create_remote_folder,
            delete_remote_folder,
            create_local_folder,
            delete_local_folder,
            start_oauth2_flow,
            listen_for_oauth_callback,
            complete_oauth2_flow,
            load_attachments_info,
            download_attachment,
            save_attachment_to_file,
            mark_email_as_read,
            mark_email_as_unread,
            mark_email_as_flagged,
            mark_email_as_unflagged,
            test_connection,
            start_idle,
            stop_idle,
            stop_all_idle,
            is_idle_active,
            start_idle_for_account,
            stop_idle_for_account,
            start_idle_for_all_accounts,
            save_draft,
            load_draft,
            list_drafts,
            delete_draft,
            get_log_directory,
            get_current_log_file,
            read_recent_logs,
            list_log_files,
            read_log_file,
            // Auth commands
            get_secure_storage,
            set_secure_storage,
            delete_secure_storage,
            sync_app_user,
            get_app_user,
            delete_app_user
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
