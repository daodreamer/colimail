#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::command;
use native_tls::TlsConnector;

// 用于存储邮件账户配置的结构体
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AccountConfig {
    email: String,
    password: String,
    imap_server: String,
    imap_port: u16,
    smtp_server: String,
    smtp_port: u16,
}

// 保存账户配置的命令
#[command]
fn save_account_config(config: AccountConfig) {
    println!("Received account config: {:?}", config);
    // 在实际应用中，这里应该将配置加密并保存到本地数据库
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
        
        let imap_session = match client_result {
            Ok(client) => client,
            Err(error) => {
                eprintln!("Error connecting to IMAP: {}", error);
                return;
            }
        };

        let mut imap_session = match imap_session.login(email, password) {
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
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_account_config,
            fetch_emails,
            send_email
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
