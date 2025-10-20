#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;

use commands::{
    fetch_email_body, fetch_emails, load_account_configs, save_account_config, send_email,
};

fn main() {
    db::init().expect("Failed to initialize database");

    match load_account_configs() {
        Ok(accounts) => println!(
            "🚀 App startup: Loaded {} accounts from database.",
            accounts.len()
        ),
        Err(e) => eprintln!("Error loading accounts on startup: {}", e),
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_account_config,
            load_account_configs,
            fetch_emails,
            fetch_email_body,
            send_email
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
