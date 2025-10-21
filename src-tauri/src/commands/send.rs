use crate::models::AccountConfig;
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

    let from: Mailbox = config.email.parse::<Mailbox>().map_err(|e| e.to_string())?;
    let to_mailbox: Mailbox = to.parse::<Mailbox>().map_err(|e| e.to_string())?;

    let email = Message::builder()
        .from(from)
        .to(to_mailbox)
        .subject(subject)
        .body(body)
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(config.email.clone(), config.password.clone());

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
