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


// --- 数据结构定义 ---
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EmailHeader {
    uid: u32,
    subject: String,
    from: String,
    date: String,
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


// 异步收取邮件
#[command]
async fn fetch_emails(config: AccountConfig) -> Result<Vec<EmailHeader>, String> {
    println!("Fetching emails for {}", config.email);
    let email_for_log = config.email.clone(); // Clone email before config is moved.
    
    let emails = tokio::task::spawn_blocking(move || -> Result<Vec<EmailHeader>, String> {
        let domain = config.imap_server.as_str();
        let port = config.imap_port;
        let email = config.email.as_str();
        let password = config.password.as_str();

        let tls = TlsConnector::builder().build().map_err(|e| e.to_string())?;
        let client = imap::connect((domain, port), domain, &tls).map_err(|e| e.to_string())?;

        let mut imap_session = client.login(email, password).map_err(|e| e.0.to_string())?;
        println!("IMAP login successful");

        let mailbox = imap_session.select("INBOX").map_err(|e| e.to_string())?;
        println!("INBOX selected with {} messages", mailbox.exists);

        let total = mailbox.exists;
        if total == 0 {
            return Ok(Vec::new());
        }

        let start = total.saturating_sub(19);
        let seq_range = format!("{}:{}", start, total);

        // 获取最近20封邮件的 ENVELOPE
        let messages = imap_session.fetch(seq_range, "(UID ENVELOPE)").map_err(|e| e.to_string())?;
        
        let mut headers = Vec::new();
        // 邮件是按顺序返回的，我们需要反转它以将最新的显示在最前面
        for msg in messages.iter().rev() { 
            let envelope = msg.envelope().ok_or("No envelope found")?;
            let subject = envelope
                .subject
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string())
                .unwrap_or_else(|| "(No Subject)".to_string());

            let from = envelope
                .from
                .as_ref()
                .map(|addrs| {
                    addrs.iter().map(|addr| {
                        format!("{}", String::from_utf8_lossy(addr.mailbox.unwrap_or_default()))
                    }).collect::<Vec<_>>().join(", ")
                })
                .unwrap_or_else(|| "(Unknown Sender)".to_string());

            let date = envelope
                .date
                .as_ref()
                .map(|d| String::from_utf8_lossy(d).to_string())
                .unwrap_or_else(|| "(No Date)".to_string());

            headers.push(EmailHeader {
                uid: msg.uid.unwrap_or(0),
                subject,
                from,
                date,
            });
        }

        let _ = imap_session.logout();
        Ok(headers)
    }).await.map_err(|e| e.to_string())??;

    println!("✅ Fetched {} email headers for {}", emails.len(), email_for_log);
    Ok(emails)
}

#[command]
async fn fetch_email_body(config: AccountConfig, uid: u32) -> Result<String, String> {
    println!("Fetching body for UID {}", uid);

    let body = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let domain = config.imap_server.as_str();
        let port = config.imap_port;
        let email = config.email.as_str();
        let password = config.password.as_str();

        let tls = TlsConnector::builder().build().map_err(|e| e.to_string())?;
        let client = imap::connect((domain, port), domain, &tls).map_err(|e| e.to_string())?;

        let mut imap_session = client.login(email, password).map_err(|e| e.0.to_string())?;
        imap_session.select("INBOX").map_err(|e| e.to_string())?;

        let messages = imap_session.uid_fetch(uid.to_string(), "BODY[]").map_err(|e| e.to_string())?;
        let message = messages.first().ok_or("No message found for UID")?;

        let raw_body = message.body().unwrap_or_default();
        
        let parsed_mail = mailparse::parse_mail(raw_body).map_err(|e| e.to_string())?;

        // 优先查找 HTML 正文，其次是纯文本正文
        let mut html_body = None;
        let mut text_body = None;

        if parsed_mail.ctype.mimetype == "text/html" {
            html_body = Some(parsed_mail.get_body().unwrap_or_default());
        } else if parsed_mail.ctype.mimetype == "text/plain" {
            text_body = Some(parsed_mail.get_body().unwrap_or_default());
        }

        for part in &parsed_mail.subparts {
            if part.ctype.mimetype == "text/html" {
                html_body = Some(part.get_body().unwrap_or_default());
                break; // 找到HTML就优先使用
            } else if part.ctype.mimetype == "text/plain" {
                text_body = Some(part.get_body().unwrap_or_default());
            }
        }

        let final_body = if let Some(body) = html_body {
            body
        } else if let Some(body) = text_body {
            // 将纯文本转换为 pre 标签，以保留换行和空格
            format!("<pre>{}</pre>", html_escape::encode_text(&body))
        } else {
            "(No readable body found)".to_string()
        };

        let _ = imap_session.logout();
        Ok(final_body)
    }).await.map_err(|e| e.to_string())??;

    println!("✅ Fetched and parsed body for UID {}", uid);
    Ok(body)
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
            fetch_email_body,
            send_email
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
