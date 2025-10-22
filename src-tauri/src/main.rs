#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod oauth2_config;

use commands::{
    complete_oauth2_flow, delete_account, delete_email, fetch_email_body, fetch_emails,
    fetch_folders, forward_email, listen_for_oauth_callback, load_account_configs, load_folders,
    move_email_to_trash, reply_email, save_account_config, send_email, start_oauth2_flow,
    sync_folders,
};

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
        .invoke_handler(tauri::generate_handler![
            save_account_config,
            load_account_configs,
            delete_account,
            fetch_emails,
            fetch_email_body,
            move_email_to_trash,
            delete_email,
            send_email,
            reply_email,
            forward_email,
            fetch_folders,
            sync_folders,
            load_folders,
            start_oauth2_flow,
            listen_for_oauth_callback,
            complete_oauth2_flow
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
