use crate::db::pool;
use crate::models::{AccountConfig, AuthType, OAuth2StartRequest, OAuth2StartResponse};
use crate::oauth2_config::OAuth2Provider;
use tauri::command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[command]
pub async fn start_oauth2_flow(request: OAuth2StartRequest) -> Result<OAuth2StartResponse, String> {
    let provider = OAuth2Provider::get_provider(&request.provider)?;
    let (auth_url, state) = provider.generate_auth_url()?;

    Ok(OAuth2StartResponse { auth_url, state })
}

#[command]
pub async fn complete_oauth2_flow(
    provider: String,
    email: String,
    code: String,
    state: String,
) -> Result<AccountConfig, String> {
    let provider_config = OAuth2Provider::get_provider(&provider)?;

    // Exchange authorization code for tokens
    let (access_token, refresh_token, expires_at) =
        provider_config.exchange_code(&code, &state).await?;

    // Create account config with OAuth2 credentials
    let account = AccountConfig {
        id: None,
        email: email.clone(),
        password: None,
        imap_server: provider_config.imap_server.clone(),
        imap_port: provider_config.imap_port,
        smtp_server: provider_config.smtp_server.clone(),
        smtp_port: provider_config.smtp_port,
        auth_type: Some(AuthType::OAuth2),
        access_token: Some(access_token.clone()),
        refresh_token,
        token_expires_at: expires_at,
    };

    // Save to database
    let pool = pool();

    sqlx::query(
        "INSERT OR REPLACE INTO accounts
         (email, password, imap_server, imap_port, smtp_server, smtp_port, auth_type, access_token, refresh_token, token_expires_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&account.email)
    .bind(&account.password)
    .bind(&account.imap_server)
    .bind(account.imap_port as i64)
    .bind(&account.smtp_server)
    .bind(account.smtp_port as i64)
    .bind("oauth2")
    .bind(&account.access_token)
    .bind(&account.refresh_token)
    .bind(account.token_expires_at)
    .execute(&*pool)
    .await
    .map_err(|e| format!("Failed to save account: {}", e))?;

    println!("✅ OAuth2 account saved to database: {}", email);
    Ok(account)
}

#[command]
pub async fn listen_for_oauth_callback() -> Result<(String, String), String> {
    let listener = TcpListener::bind("127.0.0.1:8765")
        .await
        .map_err(|e| format!("Failed to bind to port 8765: {}", e))?;

    println!("🎧 Listening for OAuth callback on http://localhost:8765");

    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| format!("Failed to accept connection: {}", e))?;

    let buf_reader = tokio::io::BufReader::new(&mut stream);
    let mut lines = buf_reader.lines();

    let request_line = lines
        .next_line()
        .await
        .map_err(|e| format!("Failed to read request line: {}", e))?
        .ok_or_else(|| "Empty request".to_string())?;

    // Parse the callback URL
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("Invalid HTTP request".to_string());
    }

    let path = parts[1];
    let url = format!("http://localhost:8765{}", path);
    let parsed_url = url::Url::parse(&url).map_err(|e| format!("Failed to parse URL: {}", e))?;

    let mut code = None;
    let mut state = None;

    for (key, value) in parsed_url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            _ => {}
        }
    }

    // Send success response to browser
    let response = "HTTP/1.1 200 OK\r\n\
                   Content-Type: text/html; charset=UTF-8\r\n\
                   \r\n\
                   <!DOCTYPE html>\
                   <html>\
                   <head><title>Authentication Successful</title></head>\
                   <body style='font-family: Arial, sans-serif; text-align: center; padding: 50px;'>\
                   <h1>✅ Authentication Successful</h1>\
                   <p>You can now close this window and return to the application.</p>\
                   </body>\
                   </html>";

    stream
        .write_all(response.as_bytes())
        .await
        .map_err(|e| format!("Failed to write response: {}", e))?;

    stream
        .flush()
        .await
        .map_err(|e| format!("Failed to flush stream: {}", e))?;

    let code = code.ok_or_else(|| "Missing authorization code".to_string())?;
    let state = state.ok_or_else(|| "Missing state parameter".to_string())?;

    Ok((code, state))
}
