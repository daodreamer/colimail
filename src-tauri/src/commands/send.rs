use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType};
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
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

    let creds = match config.auth_type {
        Some(AuthType::OAuth2) => {
            let access_token = config
                .access_token
                .clone()
                .ok_or("Access token is required for OAuth2 authentication")?;
            Credentials::new(config.email.clone(), access_token)
        }
        _ => {
            let password = config
                .password
                .clone()
                .ok_or("Password is required for basic authentication")?;
            Credentials::new(config.email.clone(), password)
        }
    };

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .port(config.smtp_port)
        .build();

    tokio::spawn(async move {
        if let Err(e) = mailer.send(email).await {
            eprintln!("Could not send email: {:?}", e);
        } else {
            println!("Email sent successfully!");
        }
    });

    Ok("Started sending email.".into())
}
