#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::command;
use native_tls::TlsConnector;
use std::sync::Mutex;

use lazy_static::lazy_static;
use directories::ProjectDirs;
use rusqlite::{Connection, Result};

// --- 数据库定义 ---
lazy_static! {
    // 使用 Mutex 来确保数据库连接的线程安全
    pub static ref DB_CONNECTION: Mutex<Connection> = {
        let proj_dirs = ProjectDirs::from("com", "MailDesk", "MailDesk").unwrap();
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).unwrap();
        let db_path = data_dir.join("maildesk.db");

        let conn = Connection::open(db_path).expect("Failed to open database");
        Mutex::new(conn)
    };
}

fn init_database() -> Result<()> {
    let conn = DB_CONNECTION.lock().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (\n            id INTEGER PRIMARY KEY,\n            email TEXT NOT NULL UNIQUE,\n            password TEXT NOT NULL, -- TODO: Encrypt this!\n            imap_server TEXT NOT NULL,\n            imap_port INTEGER NOT NULL,\n            smtp_server TEXT NOT NULL,\n            smtp_port INTEGER NOT NULL\n        )",
        (),
    )?;
    Ok(())
}


// --- 账户配置数据结构 ---
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AccountConfig {
    id: Option<i32>,
    email: String,
    password: String,
    imap_server: String,
    imap_port: u16,
    smtp_server: String,
    smtp_port: u16,
}

// --- Tauri 命令 ---

#[command]
fn save_account_config(config: AccountConfig) -> Result<(), String> {
    let conn = DB_CONNECTION.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO accounts (email, password, imap_server, imap_port, smtp_server, smtp_port) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (&config.email, &config.password, &config.imap_server, &config.imap_port, &config.smtp_server, &config.smtp_port),
    ).map_err(|e| e.to_string())?;
    println!("✅ Account saved to database: {}", config.email);
    Ok(())
}

#[command]
fn load_account_configs() -> Result<Vec<AccountConfig>, String> {
    let conn = DB_CONNECTION.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, email, password, imap_server, imap_port, smtp_server, smtp_port FROM accounts").map_err(|e| e.to_string())?;
    let accounts_iter = stmt.query_map([], |row| {
        Ok(AccountConfig {
            id: Some(row.get(0)?),
            email: row.get(1)?,
            password: row.get(2)?,
            imap_server: row.get(3)?,
            imap_port: row.get(4)?,
            smtp_server: row.get(5)?,
            smtp_port: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut accounts = Vec::new();
    for account in accounts_iter {
        accounts.push(account.map_err(|e| e.to_string())?);
    }
    Ok(accounts)
}


// 异步收取邮件的骨架
#[command]
async fn fetch_emails(config: AccountConfig) -> Result<String, String> {
    println!("Fetching emails for {}", config.email);
    
tokio::task::spawn_blocking(move || {
        let domain = config.imap_server.as_str();
        let port = config.imap_port;
        let email = config.email.as_str();
        let password = config.password.as_str();

        let tls = TlsConnector::builder().build().unwrap();
        let client_result = imap::connect((domain, port), domain, &tls);
        
        let client = match client_result {
            Ok(client) => client,
            Err(error) => {
                eprintln!("Error connecting to IMAP: {}", error);
                return;
            }
        };

        let mut imap_session = match client.login(email, password) {
            Ok(session) => session,
            Err((e, _)) => {
                eprintln!("Error logging in: {}", e);
                return;
            }
        };

        if let Err(error) = imap_session.select("INBOX") {
            eprintln!("Error selecting INBOX: {}", error);
            return;
        }
        println!("INBOX selected");

        match imap_session.fetch("1:*", "UID") {
            Ok(messages) => {
                for msg in messages.iter() {
                    println!("Message UID: {:?}", msg.uid);
                }
            },
            Err(error) => {
                eprintln!("Error fetching messages: {}", error);
                return;
            }
        }

        let _ = imap_session.logout();
    });

    Ok("Started fetching emails in background.".into())
}

// 异步发送邮件的骨架
#[command]
async fn send_email(config: AccountConfig, to: String, subject: String, body: String) -> Result<String, String> {
    println!("Sending email to {}", to);
    use lettre::{
        transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor
    };
    use lettre::message::Mailbox;

    let from: Mailbox = config.email.parse().unwrap();
    let to: Mailbox = to.parse().unwrap();

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .body(body)
        .unwrap();

    let creds = Credentials::new(config.email.clone(), config.password.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)
        .unwrap()
        .credentials(creds)
        .port(config.smtp_port)
        .build();

    tokio::spawn(async move {
        match mailer.send(email).await {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => eprintln!("Could not send email: {:?}", e),
        }
    });

    Ok("Started sending email.".into())
}


fn main() {
    // 在应用启动时初始化数据库
    init_database().expect("Failed to initialize database");

    // 启动时加载并打印现有账户，用于调试和验证
    match load_account_configs() {
        Ok(accounts) => println!("🚀 App startup: Loaded {} accounts from database.", accounts.len()),
        Err(e) => eprintln!("Error loading accounts on startup: {}", e),
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_account_config,
            load_account_configs,
            fetch_emails,
            send_email
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
