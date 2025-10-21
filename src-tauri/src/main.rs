#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod oauth2_config;

use commands::{
    complete_oauth2_flow, fetch_email_body, fetch_emails, listen_for_oauth_callback,
    load_account_configs, save_account_config, send_email, start_oauth2_flow,
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
            fetch_emails,
            fetch_email_body,
            send_email,
            start_oauth2_flow,
            listen_for_oauth_callback,
            complete_oauth2_flow
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
