use crate::models::AccountConfig;
use tauri::command;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TestConnectionResult {
    pub imap_success: bool,
    pub imap_error: Option<String>,
    pub smtp_success: bool,
    pub smtp_error: Option<String>,
}

#[command]
pub async fn test_connection(config: AccountConfig) -> Result<TestConnectionResult, String> {
    println!("🔌 Testing connection for: {}", config.email);

    // Test IMAP connection in blocking task
    let imap_result = tokio::task::spawn_blocking({
        let config = config.clone();
        move || test_imap_connection(&config)
    })
    .await
    .map_err(|e| format!("IMAP test task failed: {}", e))?;

    // Test SMTP connection (async)
    let smtp_result = test_smtp_connection(&config).await;

    Ok(TestConnectionResult {
        imap_success: imap_result.is_ok(),
        imap_error: imap_result.err(),
        smtp_success: smtp_result.is_ok(),
        smtp_error: smtp_result.err(),
    })
}

fn test_imap_connection(config: &AccountConfig) -> Result<(), String> {
    let domain = config.imap_server.as_str();
    let port = config.imap_port;
    let email = config.email.as_str();

    println!("🔌 Connecting to IMAP server: {}:{}", domain, port);

    let client = imap::ClientBuilder::new(domain, port)
        .connect()
        .map_err(|e| format!("Failed to connect to IMAP server: {}", e))?;

    // Attempt login
    let password = config
        .password
        .as_ref()
        .ok_or_else(|| "Password is required".to_string())?;

    client
        .login(email, password)
        .map_err(|e| format!("IMAP authentication failed: {}", e.0))?;

    println!("✅ IMAP connection successful");
    Ok(())
}

async fn test_smtp_connection(config: &AccountConfig) -> Result<(), String> {
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{AsyncSmtpTransport, Tokio1Executor};

    let password = config
        .password
        .as_ref()
        .ok_or_else(|| "Password is required".to_string())?;

    let creds = Credentials::new(config.email.clone(), password.clone());

    println!(
        "🔌 Connecting to SMTP server: {}:{}",
        config.smtp_server, config.smtp_port
    );

    // Use starttls_relay which matches the send.rs implementation
    // This uses STARTTLS and works with most SMTP servers
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
            .credentials(creds)
            .port(config.smtp_port)
            .build();

    // Test connection
    mailer
        .test_connection()
        .await
        .map_err(|e| format!("SMTP connection test failed: {}", e))?;

    println!("✅ SMTP connection successful");
    Ok(())
}
