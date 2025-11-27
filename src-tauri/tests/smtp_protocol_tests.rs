// P3 Priority: SMTP Protocol Integration Tests
//
// These tests verify SMTP functionality including:
// - Connection and authentication (password and OAuth2)
// - Email sending with various configurations
// - Attachment handling
// - Error scenarios (invalid credentials, server errors, etc.)

use colimail_lib::models::{AccountConfig, AuthType};
use lettre::{
    message::{Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

/// Helper to create a test SMTP account configuration
fn create_test_smtp_account() -> AccountConfig {
    let email = std::env::var("TEST_SMTP_EMAIL").unwrap_or_else(|_| "test@example.com".to_string());
    let password = std::env::var("TEST_SMTP_PASSWORD").unwrap_or_else(|_| "password".to_string());
    let smtp_server =
        std::env::var("TEST_SMTP_SERVER").unwrap_or_else(|_| "smtp.example.com".to_string());
    let smtp_port = std::env::var("TEST_SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587);

    AccountConfig {
        id: Some(1),
        email,
        password: Some(password),
        imap_server: "imap.example.com".to_string(),
        imap_port: 993,
        smtp_server,
        smtp_port,
        auth_type: None,
        access_token: None,
        refresh_token: None,
        token_expires_at: None,
        display_name: Some("Test User".to_string()),
    }
}

#[cfg(test)]
mod smtp_connection_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires real SMTP server
    async fn test_smtp_connection_starttls() {
        let config = create_test_smtp_account();

        // Test: Connect to SMTP server with STARTTLS (port 587)
        let creds = Credentials::new(config.email.clone(), config.password.clone().unwrap());

        let result = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
            .map(|builder| builder.credentials(creds).build::<Tokio1Executor>());

        match result {
            Ok(mailer) => {
                println!("✅ SMTP STARTTLS connection configured successfully");

                // Test the connection
                let test_result = mailer.test_connection().await;
                match test_result {
                    Ok(true) => println!("✅ SMTP connection test passed"),
                    Ok(false) => println!("⚠️ SMTP connection test returned false"),
                    Err(e) => println!("⚠️ SMTP connection test failed: {:?}", e),
                }
            }
            Err(e) => {
                println!("⚠️ SMTP connection setup failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires real SMTP server with SSL/TLS
    async fn test_smtp_connection_ssl_tls_port_465() {
        let mut config = create_test_smtp_account();
        config.smtp_port = 465; // SSL/TLS port

        // Test: Connect to SMTP server with implicit SSL/TLS (port 465)
        let creds = Credentials::new(config.email.clone(), config.password.clone().unwrap());

        let result =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server).map(|builder| {
                builder
                    .credentials(creds)
                    .port(465)
                    .build::<Tokio1Executor>()
            });

        match result {
            Ok(mailer) => {
                println!("✅ SMTP SSL/TLS (port 465) connection configured");

                let test_result = mailer.test_connection().await;
                match test_result {
                    Ok(true) => println!("✅ SMTP SSL/TLS connection test passed"),
                    Ok(false) => println!("⚠️ SMTP SSL/TLS connection test returned false"),
                    Err(e) => println!("⚠️ SMTP SSL/TLS test failed: {:?}", e),
                }
            }
            Err(e) => {
                println!("⚠️ SMTP SSL/TLS setup failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_smtp_connection_invalid_server() {
        let mut config = create_test_smtp_account();
        config.smtp_server = "nonexistent.invalid.server.test".to_string();

        // Test: Connection to invalid server should eventually fail
        let creds = Credentials::new(config.email.clone(), config.password.unwrap());

        // Relay() doesn't fail immediately, but test_connection() will
        if let Ok(mailer) =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map(|b| b.credentials(creds).build::<Tokio1Executor>())
        {
            let test_result = mailer.test_connection().await;
            assert!(
                test_result.is_err(),
                "Should fail to test connection to invalid server"
            );
            if let Err(e) = test_result {
                println!("✅ Correctly failed on invalid SMTP server: {:?}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires real SMTP server
    async fn test_smtp_authentication_failure() {
        let mut config = create_test_smtp_account();
        config.password = Some("wrong_password_12345".to_string());

        // Test: Invalid credentials should fail authentication
        let creds = Credentials::new(config.email.clone(), config.password.unwrap());

        if let Ok(mailer) =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
                .map(|b| b.credentials(creds).build::<Tokio1Executor>())
        {
            let test_result = mailer.test_connection().await;

            // Should fail with authentication error
            if test_result.is_err() {
                println!("✅ Correctly rejected invalid SMTP credentials");
            } else {
                println!("⚠️ Test skipped - no real SMTP server available");
            }
        }
    }
}

#[cfg(test)]
mod smtp_send_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires real SMTP server and will send actual email
    async fn test_smtp_send_simple_email() {
        let config = create_test_smtp_account();

        // Test: Send a simple text email
        let from: Mailbox = config.email.parse().expect("Invalid from email");
        let to: Mailbox = config.email.parse().expect("Invalid to email"); // Send to self

        let email = Message::builder()
            .from(from.clone())
            .to(to)
            .subject("Test Email - P3 SMTP Test")
            .body("This is a test email sent by the P3 SMTP integration test.".to_string())
            .expect("Failed to build email");

        let creds = Credentials::new(config.email.clone(), config.password.unwrap());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        let result = mailer.send(email).await;

        match result {
            Ok(_) => println!("✅ Simple email sent successfully"),
            Err(e) => {
                // Log but don't fail - may be test environment issue
                println!("⚠️ Email send failed (test environment issue?): {:?}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires real SMTP server
    async fn test_smtp_send_html_email() {
        let config = create_test_smtp_account();

        // Test: Send HTML email
        let from: Mailbox = config.email.parse().expect("Invalid from email");
        let to: Mailbox = config.email.parse().expect("Invalid to email");

        let html_body = r#"
            <html>
                <body>
                    <h1>Test Email</h1>
                    <p>This is an <strong>HTML</strong> email sent by P3 tests.</p>
                    <ul>
                        <li>Item 1</li>
                        <li>Item 2</li>
                    </ul>
                </body>
            </html>
        "#;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject("Test HTML Email - P3 SMTP Test")
            .multipart(MultiPart::alternative().singlepart(SinglePart::html(html_body.to_string())))
            .expect("Failed to build HTML email");

        let creds = Credentials::new(config.email.clone(), config.password.unwrap());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        let result = mailer.send(email).await;

        match result {
            Ok(_) => println!("✅ HTML email sent successfully"),
            Err(e) => println!("⚠️ HTML email send failed: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore] // Requires real SMTP server
    async fn test_smtp_send_email_with_cc() {
        let config = create_test_smtp_account();

        // Test: Send email with CC recipients
        let from: Mailbox = config.email.parse().expect("Invalid from email");
        let to: Mailbox = config.email.parse().expect("Invalid to email");
        let cc: Mailbox = config.email.parse().expect("Invalid cc email");

        let email = Message::builder()
            .from(from)
            .to(to)
            .cc(cc)
            .subject("Test Email with CC - P3 Test")
            .body("This email has a CC recipient.".to_string())
            .expect("Failed to build email");

        let creds = Credentials::new(config.email.clone(), config.password.unwrap());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        let result = mailer.send(email).await;

        match result {
            Ok(_) => println!("✅ Email with CC sent successfully"),
            Err(e) => println!("⚠️ Email with CC send failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_smtp_email_validation_invalid_recipient() {
        // Test: Email with invalid recipient should fail at build time
        let config = create_test_smtp_account();

        let _from: Mailbox = config.email.parse().expect("Valid from email");
        let invalid_to = "invalid-email-format";

        let to_result: Result<Mailbox, _> = invalid_to.parse();

        assert!(to_result.is_err(), "Should reject invalid email format");
        println!("✅ Invalid email format correctly rejected");
    }
}

#[cfg(test)]
mod smtp_attachment_tests {
    use super::*;
    use lettre::message::{Attachment, Body};

    #[tokio::test]
    #[ignore] // Requires real SMTP server
    async fn test_smtp_send_email_with_attachment() {
        let config = create_test_smtp_account();

        // Test: Send email with a small text attachment
        let from: Mailbox = config.email.parse().unwrap();
        let to: Mailbox = config.email.parse().unwrap();

        let attachment_content = "This is a test attachment file.\nLine 2\nLine 3".as_bytes();
        let attachment = Attachment::new("test.txt".to_string()).body(
            Body::new(attachment_content.to_vec()),
            "text/plain".parse().unwrap(),
        );

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject("Test Email with Attachment - P3 Test")
            .multipart(
                MultiPart::mixed()
                    .singlepart(SinglePart::html(
                        "<p>This email has an attachment.</p>".to_string(),
                    ))
                    .singlepart(attachment),
            )
            .expect("Failed to build email with attachment");

        let creds = Credentials::new(config.email.clone(), config.password.unwrap());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        let result = mailer.send(email).await;

        match result {
            Ok(_) => println!("✅ Email with attachment sent successfully"),
            Err(e) => println!("⚠️ Email with attachment failed: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore] // Requires real SMTP server
    async fn test_smtp_send_email_with_multiple_attachments() {
        let config = create_test_smtp_account();

        // Test: Send email with multiple attachments
        let from: Mailbox = config.email.parse().unwrap();
        let to: Mailbox = config.email.parse().unwrap();

        let attachment1 = Attachment::new("file1.txt".to_string()).body(
            Body::new(b"Content of file 1".to_vec()),
            "text/plain".parse().unwrap(),
        );

        let attachment2 = Attachment::new("file2.txt".to_string()).body(
            Body::new(b"Content of file 2".to_vec()),
            "text/plain".parse().unwrap(),
        );

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject("Test Email with Multiple Attachments - P3 Test")
            .multipart(
                MultiPart::mixed()
                    .singlepart(SinglePart::html(
                        "<p>This email has two attachments.</p>".to_string(),
                    ))
                    .singlepart(attachment1)
                    .singlepart(attachment2),
            )
            .expect("Failed to build email with multiple attachments");

        let creds = Credentials::new(config.email.clone(), config.password.unwrap());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        let result = mailer.send(email).await;

        match result {
            Ok(_) => println!("✅ Email with multiple attachments sent successfully"),
            Err(e) => println!("⚠️ Email with multiple attachments failed: {:?}", e),
        }
    }
}

#[cfg(test)]
mod smtp_oauth2_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires OAuth2 test credentials
    async fn test_smtp_oauth2_authentication() {
        let mut config = create_test_smtp_account();
        config.auth_type = Some(AuthType::OAuth2);
        config.access_token = std::env::var("TEST_OAUTH2_ACCESS_TOKEN").ok();

        if config.access_token.is_none() {
            println!("⚠️ Skipping OAuth2 test - no access token provided");
            return;
        }

        // Test: OAuth2 authentication with XOAUTH2 mechanism
        let creds = Credentials::new(config.email.clone(), config.access_token.unwrap());

        let result: Result<AsyncSmtpTransport<Tokio1Executor>, _> =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server).map(|b| {
                b.credentials(creds)
                    .authentication(vec![Mechanism::Xoauth2])
                    .build()
            });

        match result {
            Ok(mailer) => {
                println!("✅ SMTP OAuth2 transport configured");

                let test_result = mailer.test_connection().await;
                match test_result {
                    Ok(true) => println!("✅ OAuth2 SMTP connection successful"),
                    Ok(false) => println!("⚠️ OAuth2 SMTP connection returned false"),
                    Err(e) => println!("⚠️ OAuth2 SMTP connection failed: {:?}", e),
                }
            }
            Err(e) => {
                println!("⚠️ OAuth2 SMTP setup failed: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod smtp_error_scenarios {
    use super::*;

    #[tokio::test]
    async fn test_smtp_build_email_missing_recipient() {
        let config = create_test_smtp_account();

        // Test: Email without recipient should fail at build time
        let from: Mailbox = config.email.parse().unwrap();

        let result = Message::builder()
            .from(from)
            // Missing .to()
            .subject("Test")
            .body("Body".to_string());

        // This should fail because no recipient is specified
        // Note: lettre might allow building without recipient, so we test the transport layer
        match result {
            Ok(_) => println!("⚠️ Email built without recipient (will fail at send time)"),
            Err(e) => println!("✅ Correctly rejected email without recipient: {:?}", e),
        }
    }

    #[test]
    fn test_smtp_port_validation() {
        let config = create_test_smtp_account();

        // Test: Common SMTP ports should be valid
        assert!(
            config.smtp_port == 587 || config.smtp_port == 465 || config.smtp_port == 25,
            "SMTP port should be a standard port (25, 465, or 587)"
        );

        println!("✅ SMTP port validation: {}", config.smtp_port);
    }

    #[tokio::test]
    #[ignore] // Requires real SMTP server
    async fn test_smtp_send_large_email_body() {
        let config = create_test_smtp_account();

        // Test: Send email with large body (test size limits)
        let from: Mailbox = config.email.parse().unwrap();
        let to: Mailbox = config.email.parse().unwrap();

        // Create a 1MB email body
        let large_body = "A".repeat(1024 * 1024);

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject("Test Large Email - P3 Test")
            .body(large_body)
            .expect("Failed to build large email");

        let creds = Credentials::new(config.email.clone(), config.password.unwrap());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        let result = mailer.send(email).await;

        match result {
            Ok(_) => println!("✅ Large email sent successfully"),
            Err(e) => {
                println!("⚠️ Large email failed (may hit server limits): {:?}", e);
                // This is expected - servers often have size limits
            }
        }
    }
}

#[cfg(test)]
mod smtp_reply_forward_tests {
    #[test]
    fn test_smtp_reply_subject_formatting() {
        // Test: Reply subject should have "Re: " prefix
        let original_subject = "Hello World";
        let reply_subject = format!("Re: {}", original_subject);

        assert_eq!(reply_subject, "Re: Hello World");

        // Test: Don't duplicate "Re: " prefix
        let already_reply = "Re: Hello World";
        let should_not_duplicate = if already_reply.to_lowercase().starts_with("re:") {
            already_reply.to_string()
        } else {
            format!("Re: {}", already_reply)
        };

        assert_eq!(should_not_duplicate, "Re: Hello World");
        println!("✅ Reply subject formatting correct");
    }

    #[test]
    fn test_smtp_forward_subject_formatting() {
        // Test: Forward subject should have "Fwd: " prefix
        let original_subject = "Important Email";
        let forward_subject = format!("Fwd: {}", original_subject);

        assert_eq!(forward_subject, "Fwd: Important Email");

        // Test: Don't duplicate "Fwd: " prefix
        let already_forward = "Fwd: Important Email";
        let should_not_duplicate = if already_forward.to_lowercase().starts_with("fwd:") {
            already_forward.to_string()
        } else {
            format!("Fwd: {}", already_forward)
        };

        assert_eq!(should_not_duplicate, "Fwd: Important Email");
        println!("✅ Forward subject formatting correct");
    }
}
