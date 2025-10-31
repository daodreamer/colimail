use crate::commands::emails::fetch::OAuth2;
use crate::commands::utils::ensure_valid_token;
use crate::models::{AccountConfig, AuthType};
use mail_parser::MessageParser;
use tauri::command;

#[command]
pub async fn detect_display_name_from_sent(
    config: AccountConfig,
) -> Result<Option<String>, String> {
    println!(
        "🔍 Detecting display name from Sent folder for {}",
        config.email
    );

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    // Connect to IMAP server
    let result = tokio::task::spawn_blocking(move || -> Result<Option<String>, String> {
        // Use ClientBuilder as per imap 3.0.0 API
        let domain = config.imap_server.as_str();
        let port = config.imap_port;

        let client = imap::ClientBuilder::new(domain, port)
            .connect()
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Authenticate
        let mut session = authenticate_imap(client, &config)?;

        // Try to find Sent folder
        let sent_folder = find_sent_folder(&mut session)?;

        if sent_folder.is_none() {
            println!("⚠️  No Sent folder found");
            return Ok(None);
        }

        let sent_folder = sent_folder.unwrap();
        println!("📁 Found Sent folder: {}", sent_folder);

        // Select Sent folder
        session
            .select(&sent_folder)
            .map_err(|e| format!("Failed to select Sent folder: {}", e))?;

        // Search for the most recent emails (limit to last 5)
        let search_result = session
            .search("ALL")
            .map_err(|e| format!("Failed to search emails: {}", e))?;

        if search_result.is_empty() {
            println!("ℹ️  No emails found in Sent folder");
            return Ok(None);
        }

        // Get the last 5 emails (or fewer if there are less than 5)
        // Convert HashSet to Vec and sort to get consistent ordering
        let mut uid_vec: Vec<u32> = search_result.iter().copied().collect();
        uid_vec.sort_unstable();

        let recent_uids: Vec<u32> = uid_vec.iter().rev().take(5).copied().collect();

        println!("📧 Checking {} recent sent emails", recent_uids.len());

        // Try to extract display name from recent emails
        for uid in recent_uids {
            if let Some(display_name) = extract_display_name_from_email(&mut session, uid)? {
                println!("✅ Found display name: {}", display_name);
                session.logout().ok();
                return Ok(Some(display_name));
            }
        }

        println!("ℹ️  No display name found in recent sent emails");
        session.logout().ok();
        Ok(None)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

fn authenticate_imap(
    client: imap::Client<Box<dyn imap::ImapConnection>>,
    config: &AccountConfig,
) -> Result<imap::Session<Box<dyn imap::ImapConnection>>, String> {
    match config.auth_type {
        Some(AuthType::OAuth2) => {
            let access_token = config
                .access_token
                .as_ref()
                .ok_or("Access token is required for OAuth2")?;

            let oauth2 = OAuth2 {
                user: config.email.clone(),
                access_token: access_token.clone(),
            };

            client
                .authenticate("XOAUTH2", &oauth2)
                .map_err(|e| format!("OAuth2 authentication failed: {}", e.0))
        }
        _ => {
            let password = config
                .password
                .as_ref()
                .ok_or("Password is required for basic authentication")?;

            client
                .login(&config.email, password)
                .map_err(|e| format!("Login failed: {}", e.0))
        }
    }
}

fn find_sent_folder(
    session: &mut imap::Session<Box<dyn imap::ImapConnection>>,
) -> Result<Option<String>, String> {
    let folders = session
        .list(Some(""), Some("*"))
        .map_err(|e| format!("Failed to list folders: {}", e))?;

    // Common sent folder names (case-insensitive)
    let sent_patterns = vec![
        "sent",
        "sent items",
        "sent mail",
        "已发送",
        "已发送邮件",
        "送信済み",
        "envoyés",
        "enviados",
        "gesendete",
        "gesendet", // German
        "inviati",  // Italian
        "enviado",  // Spanish
        "skickat",  // Swedish
    ];

    // First, try to find folder with \Sent attribute/flag
    for mailbox in folders.iter() {
        let attributes = format!("{:?}", mailbox.attributes());
        if attributes.to_lowercase().contains("sent") {
            let folder_name = mailbox.name();
            println!("📁 Found Sent folder by attribute: {}", folder_name);
            return Ok(Some(folder_name.to_string()));
        }
    }

    // If no folder with \Sent attribute, fall back to name matching
    for mailbox in folders.iter() {
        let folder_name = mailbox.name();
        let folder_lower = folder_name.to_lowercase();

        for pattern in &sent_patterns {
            if folder_lower.contains(pattern) {
                println!("📁 Found Sent folder by name pattern: {}", folder_name);
                return Ok(Some(folder_name.to_string()));
            }
        }
    }

    Ok(None)
}

fn extract_display_name_from_email(
    session: &mut imap::Session<Box<dyn imap::ImapConnection>>,
    uid: u32,
) -> Result<Option<String>, String> {
    // Fetch the email header
    let messages = session
        .uid_fetch(uid.to_string(), "BODY[HEADER.FIELDS (FROM)]")
        .map_err(|e| format!("Failed to fetch email: {}", e))?;

    if let Some(message) = messages.iter().next() {
        if let Some(body) = message.header() {
            // Parse the email header
            let parser = MessageParser::default();
            if let Some(parsed) = parser.parse(body) {
                // Extract From header
                if let Some(from) = parsed.from() {
                    if let Some(address) = from.first() {
                        if let Some(name) = address.name() {
                            return Ok(Some(name.to_string()));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}
