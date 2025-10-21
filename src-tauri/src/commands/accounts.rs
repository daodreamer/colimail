use crate::db::pool;
use crate::models::AccountConfig;
use tauri::command;

#[command]
pub async fn save_account_config(config: AccountConfig) -> Result<(), String> {
    let pool = pool();

    sqlx::query(
        "INSERT OR REPLACE INTO accounts (email, password, imap_server, imap_port, smtp_server, smtp_port)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&config.email)
    .bind(&config.password)
    .bind(&config.imap_server)
    .bind(config.imap_port as i64)
    .bind(&config.smtp_server)
    .bind(config.smtp_port as i64)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    println!("✅ Account saved to database: {}", config.email);
    Ok(())
}

#[command]
pub async fn load_account_configs() -> Result<Vec<AccountConfig>, String> {
    let pool = pool();

    let accounts = sqlx::query_as::<_, (i64, String, String, String, i64, String, i64)>(
        "SELECT id, email, password, imap_server, imap_port, smtp_server, smtp_port FROM accounts",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .map(
        |(id, email, password, imap_server, imap_port, smtp_server, smtp_port)| AccountConfig {
            id: Some(id as i32),
            email,
            password,
            imap_server,
            imap_port: imap_port as u16,
            smtp_server,
            smtp_port: smtp_port as u16,
        },
    )
    .collect();

    Ok(accounts)
}
