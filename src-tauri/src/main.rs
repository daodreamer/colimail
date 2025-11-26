#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod attachment_limits;
mod cmvh;
mod commands;
mod db;
mod encryption;
mod ens;
mod idle_commands;
mod idle_manager;
mod logger;
mod models;
mod oauth2_config;
mod security;
mod setup;
mod tray;

use commands::{
    change_master_password, check_folder_capabilities, cleanup_cmvh_cache, cleanup_ens_cache,
    clear_cmvh_cache, clear_ens_cache, complete_oauth2_flow, create_local_folder,
    create_remote_folder, delete_account, delete_app_user, delete_draft, delete_email,
    delete_local_folder, delete_remote_folder, delete_secure_storage, delete_wallet_session,
    derive_eth_address, detect_display_name_from_sent, disable_encryption, download_attachment,
    enable_encryption, export_logs_as_zip, fetch_email_body, fetch_email_body_cached,
    fetch_email_raw_headers, fetch_emails, fetch_folders, forward_email, get_all_reward_caches,
    get_app_user, get_attachment_size_limit, get_cmvh_cache, get_cmvh_cache_stats,
    get_current_log_file, get_encryption_status, get_ens_cache, get_ens_cache_stats,
    get_last_sync_time, get_log_directory, get_minimize_to_tray, get_notification_enabled,
    get_reward_cache, get_secure_storage, get_sound_enabled, get_sync_interval, get_wallet_session,
    get_wallet_session_timeout, has_cmvh_headers, hash_email_content, list_drafts, list_log_files,
    listen_for_oauth_callback, load_account_configs, load_attachments_info, load_draft,
    load_emails_from_cache, load_folders, lock_encryption_command, mark_email_as_flagged,
    mark_email_as_read, mark_email_as_unflagged, mark_email_as_unread, move_email_to_trash,
    parse_email_cmvh_headers, read_log_file, read_recent_logs, reply_email, save_account_config,
    save_attachment_to_file, save_cmvh_cache, save_draft, save_ens_cache, save_reward_cache,
    save_wallet_session, send_email, send_email_smtp, send_email_with_cmvh, set_minimize_to_tray,
    set_notification_enabled, set_secure_storage, set_sound_enabled, set_sync_interval,
    set_wallet_session_timeout, should_sync, sign_email_cmvh, sign_email_with_cmvh,
    start_oauth2_flow, sync_app_user, sync_email_flags, sync_emails, sync_folders,
    sync_specific_email_flags, test_connection, unlock_encryption_with_password,
    update_wallet_session_activity, verify_cmvh_signature,
};
use idle_commands::{
    is_idle_active, start_idle, start_idle_for_account, start_idle_for_all_accounts, stop_all_idle,
    stop_idle, stop_idle_for_account,
};
use idle_manager::IdleManager;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[tokio::main]
async fn main() {
    // Initialize logging system first
    logger::init().expect("Failed to initialize logging system");

    tracing::info!("Starting Colimail application");

    // Initialize database (metadata only, no sensitive data access yet)
    db::init().await.expect("Failed to initialize database");

    // Note: Account loading and IDLE manager startup are deferred until after user unlocks encryption
    // This ensures zero-knowledge security: no sensitive data is accessed before master password is entered

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            setup::setup_single_instance_callback(app, args);
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize application setup (deep links, notifications, window handlers, updates)
            setup::init_app(app)?;

            // Setup system tray
            tray::setup_tray(app)?;

            // Initialize IDLE manager
            let idle_manager = Arc::new(Mutex::new(Some(IdleManager::new(app.handle().clone()))));
            app.manage(idle_manager.clone());

            tracing::info!("IDLE manager initialized");

            // Note: IDLE manager auto-start has been removed for security
            // IDLE connections will be started by frontend after user unlocks encryption
            // This prevents any network activity before master password is entered

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
            fetch_email_raw_headers,
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
            export_logs_as_zip,
            // Auth commands
            get_secure_storage,
            set_secure_storage,
            delete_secure_storage,
            sync_app_user,
            get_app_user,
            delete_app_user,
            // Encryption commands
            get_encryption_status,
            enable_encryption,
            disable_encryption,
            unlock_encryption_with_password,
            lock_encryption_command,
            change_master_password,
            // CMVH commands
            parse_email_cmvh_headers,
            verify_cmvh_signature,
            hash_email_content,
            has_cmvh_headers,
            sign_email_with_cmvh,
            sign_email_cmvh,
            derive_eth_address,
            send_email_with_cmvh,
            send_email_smtp,
            // CMVH cache commands
            get_cmvh_cache,
            save_cmvh_cache,
            cleanup_cmvh_cache,
            clear_cmvh_cache,
            get_cmvh_cache_stats,
            // ENS cache commands
            get_ens_cache,
            save_ens_cache,
            cleanup_ens_cache,
            clear_ens_cache,
            get_ens_cache_stats,
            // Reward cache commands
            get_reward_cache,
            save_reward_cache,
            get_all_reward_caches,
            // Wallet security commands
            get_wallet_session_timeout,
            set_wallet_session_timeout,
            save_wallet_session,
            get_wallet_session,
            delete_wallet_session,
            update_wallet_session_activity
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
