use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType};
use lettre::{
    message::Mailbox,
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tauri::command;

#[command]
pub async fn send_email(
    config: AccountConfig,
    to: String,
    subject: String,
    body: String,
) -> Result<String, String> {
    println!("Sending email to {}", to);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    let from: Mailbox = config.email.parse::<Mailbox>().map_err(|e| e.to_string())?;
    let to_mailbox: Mailbox = to.parse::<Mailbox>().map_err(|e| e.to_string())?;

    let email = Message::builder()
        .from(from)
        .to(to_mailbox)
        .subject(subject)
        .body(body)
        .map_err(|e| e.to_string())?;

    let mailer = match config.auth_type {
        Some(AuthType::OAuth2) => {
            let access_token = config
                .access_token
                .clone()
                .ok_or("Access token is required for OAuth2 authentication")?;

            println!(
                "🔐 Building SMTP transport with XOAUTH2 for {}",
                config.email
            );
            println!("   Server: {}:{}", config.smtp_server, config.smtp_port);
            println!("   Token length: {} chars", access_token.len());

            // For OAuth2, we need to use XOAUTH2 mechanism
            // The credentials format for XOAUTH2 is: email as username, access_token as password
            let creds = Credentials::new(config.email.clone(), access_token);

            // Use starttls_relay for port 587 (STARTTLS), relay for port 465 (implicit TLS)
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map_err(|e| e.to_string())?
                .credentials(creds)
                .authentication(vec![Mechanism::Xoauth2])
                .build()
        }
        _ => {
            let password = config
                .password
                .clone()
                .ok_or("Password is required for basic authentication")?;

            let creds = Credentials::new(config.email.clone(), password);

            // Use starttls_relay for most SMTP servers
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map_err(|e| e.to_string())?
                .credentials(creds)
                .build()
        }
    };

    tokio::spawn(async move {
        if let Err(e) = mailer.send(email).await {
            eprintln!("Could not send email: {:?}", e);
        } else {
            println!("Email sent successfully!");
        }
    });

    Ok("Started sending email.".into())
}
