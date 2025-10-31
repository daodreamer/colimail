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
    tracing::info!(email = %config.email, "Testing connection");

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

    tracing::info!(server = %domain, port = port, "Connecting to IMAP server");

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

    tracing::info!("IMAP connection successful");
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

    tracing::info!(
        server = %config.smtp_server,
        port = config.smtp_port,
        "Connecting to SMTP server"
    );

    // Choose connection method based on port:
    // - Port 465: SSL/TLS (implicit TLS, used by 163.com, QQ, etc.)
    // - Port 587: STARTTLS (explicit TLS, used by Gmail, Outlook, etc.)
    // - Port 25: Plain or STARTTLS (legacy, rarely used)
    let mailer: AsyncSmtpTransport<Tokio1Executor> = if config.smtp_port == 465 {
        // Port 465 requires SSL/TLS direct connection (implicit TLS)
        tracing::debug!("Using SSL/TLS (implicit TLS) for port 465");
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
            .credentials(creds)
            .port(config.smtp_port)
            .build()
    } else {
        // Port 587 or others use STARTTLS
        tracing::debug!(port = config.smtp_port, "Using STARTTLS");
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
            .credentials(creds)
            .port(config.smtp_port)
            .build()
    };

    // Test connection
    mailer
        .test_connection()
        .await
        .map_err(|e| format!("SMTP connection test failed: {}", e))?;

    tracing::info!("SMTP connection successful");
    Ok(())
}
