use crate::db::connection;
use crate::models::AccountConfig;
use tauri::command;

#[command]
pub fn save_account_config(config: AccountConfig) -> Result<(), String> {
    let conn = connection();
    conn.execute(
        "INSERT OR REPLACE INTO accounts (email, password, imap_server, imap_port, smtp_server, smtp_port)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &config.email,
            &config.password,
            &config.imap_server,
            &config.imap_port,
            &config.smtp_server,
            &config.smtp_port,
        ),
    )
    .map_err(|e| e.to_string())?;

    println!("✅ Account saved to database: {}", config.email);
    Ok(())
}

#[command]
pub fn load_account_configs() -> Result<Vec<AccountConfig>, String> {
    let conn = connection();
    let mut stmt = conn
        .prepare(
            "SELECT id, email, password, imap_server, imap_port, smtp_server, smtp_port FROM accounts",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(AccountConfig {
                id: Some(row.get(0)?),
                email: row.get(1)?,
                password: row.get(2)?,
                imap_server: row.get(3)?,
                imap_port: row.get(4)?,
                smtp_server: row.get(5)?,
                smtp_port: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut accounts = Vec::new();
    for account in rows {
        accounts.push(account.map_err(|e| e.to_string())?);
    }

    Ok(accounts)
}
