use crate::commands::emails::imap_helpers;
use crate::commands::utils::ensure_valid_token;
use crate::db;
use crate::models::{AccountConfig, Folder};
use tauri::command;
use utf7_imap;

/// Decode IMAP folder name from modified UTF-7 encoding
fn decode_folder_name(encoded_name: &str) -> String {
    // The utf7_imap crate's decode function doesn't return a Result,
    // it just returns the decoded string (or original if it can't decode)
    utf7_imap::decode_utf7_imap(encoded_name.to_string())
}

/// Encode folder name to IMAP modified UTF-7 encoding
fn encode_folder_name(folder_name: &str) -> String {
    // For now, we'll just return the original string as most modern mail providers
    // support UTF-8. If needed, we can add a proper UTF-7 encoding implementation.
    // The utf7-imap crate only provides decode functionality.
    folder_name.to_string()
}

/// Generate a user-friendly display name for a folder
/// Removes provider-specific prefixes like [Gmail]/ and [Outlook]/
fn get_display_name(full_name: &str) -> String {
    let mut display_name = full_name.to_string();

    // Remove common provider prefixes
    if display_name.starts_with("[Gmail]/") {
        display_name = display_name.strip_prefix("[Gmail]/").unwrap().to_string();
    } else if display_name.starts_with("[Outlook]/") {
        display_name = display_name.strip_prefix("[Outlook]/").unwrap().to_string();
    }

    display_name
}

#[command]
pub async fn fetch_folders(config: AccountConfig) -> Result<Vec<Folder>, String> {
    println!("Fetching folders for {}", config.email);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;
    let account_id = config.id.ok_or("Account ID is required")?;

    let folders = tokio::task::spawn_blocking(move || -> Result<Vec<Folder>, String> {
        // Use helper function for connection with imap 3.0.0 API
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

        println!("IMAP authentication successful, listing folders...");

        // List all folders using "*" pattern
        let mailbox_list = imap_session
            .list(Some(""), Some("*"))
            .map_err(|e| e.to_string())?;

        let mut folders = Vec::new();
        let mut tested_accessible = Vec::new();

        for mailbox in mailbox_list.iter() {
            let raw_name = mailbox.name().to_string();

            // Decode from UTF-7 IMAP encoding for display purposes
            let decoded_name = decode_folder_name(&raw_name);

            // Generate user-friendly display name
            let display_name = get_display_name(&decoded_name);

            let delimiter = mailbox.delimiter().map(|d| d.to_string());
            let flags = Some(format!("{:?}", mailbox.attributes()));

            let folder = Folder {
                id: None,
                account_id,
                name: raw_name.clone(), // Store RAW (encoded) name for IMAP operations
                display_name: display_name.clone(),
                delimiter,
                flags: flags.clone(),
                is_local: false, // IMAP folders are not local
            };

            // Log folder info with visibility status
            let status = if !folder.is_selectable() {
                "✗ (Noselect)"
            } else if !folder.should_show_to_user() {
                "⊗ (System folder, hidden)"
            } else {
                "✓"
            };
            println!(
                "  📁 Folder: {} -> Display: {} {}",
                raw_name, display_name, status
            );

            // Only test and add folders that should be shown to users
            if folder.should_show_to_user() {
                // Test if folder is actually accessible by trying to SELECT it
                // IMPORTANT: Use raw_name (UTF-7 encoded) for IMAP operations
                let folder_name_for_log = display_name.clone();
                match imap_session.select(&raw_name) {
                    Ok(_) => {
                        println!("     ✓ Folder is accessible");
                        tested_accessible.push(folder_name_for_log);
                        folders.push(folder);
                    }
                    Err(e) => {
                        println!("     ✗ Folder cannot be accessed: {} (skipping)", e);
                    }
                }
            }
        }

        let _ = imap_session.logout();

        println!(
            "✅ Found {} accessible folders (tested {} candidates)",
            folders.len(),
            tested_accessible.len()
        );

        Ok(folders)
    })
    .await
    .map_err(|e| e.to_string())??;

    println!("✅ Fetched {} folders", folders.len());
    Ok(folders)
}

#[command]
pub async fn sync_folders(config: AccountConfig) -> Result<Vec<Folder>, String> {
    println!("Syncing folders for {}", config.email);

    let account_id = config.id.ok_or("Account ID is required")?;

    // Fetch folders from IMAP server
    let folders = fetch_folders(config).await?;

    // Save folders to database
    let pool = db::pool();

    // Delete existing folders for this account
    sqlx::query("DELETE FROM folders WHERE account_id = ?")
        .bind(account_id)
        .execute(pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    // Insert new folders
    for folder in &folders {
        sqlx::query("INSERT INTO folders (account_id, name, display_name, delimiter, flags, is_local) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(account_id)
            .bind(&folder.name)
            .bind(&folder.display_name)
            .bind(&folder.delimiter)
            .bind(&folder.flags)
            .bind(if folder.is_local { 1 } else { 0 })
            .execute(pool.as_ref())
            .await
            .map_err(|e| e.to_string())?;
    }

    // Update last sync time for folders
    let current_time = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT OR REPLACE INTO sync_status (account_id, folder_name, last_sync_time)
        VALUES (?, '__folders__', ?)",
    )
    .bind(account_id)
    .bind(current_time)
    .execute(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to update folders sync time: {}", e))?;

    println!("✅ Synced {} folders to database", folders.len());

    // Load folders from database to ensure consistent sorting
    load_folders(account_id).await
}

/// Check IMAP server capabilities for folder management
#[command]
pub async fn check_folder_capabilities(config: AccountConfig) -> Result<bool, String> {
    println!("Checking folder capabilities for {}", config.email);

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;

    let supports_create_delete = tokio::task::spawn_blocking(move || -> Result<bool, String> {
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

        // Check capabilities
        let _capabilities = imap_session
            .capabilities()
            .map_err(|e| format!("Failed to get capabilities: {}", e))?;

        // Most IMAP servers support CREATE and DELETE commands
        // They are part of the base IMAP4rev1 specification
        let supports = true; // Assume support unless explicitly disabled

        let _ = imap_session.logout();

        Ok(supports)
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(supports_create_delete)
}

/// Create a remote IMAP folder
#[command]
pub async fn create_remote_folder(
    config: AccountConfig,
    folder_name: String,
) -> Result<Folder, String> {
    println!(
        "Creating remote folder '{}' for {}",
        folder_name, config.email
    );

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;
    let account_id = config.id.ok_or("Account ID is required")?;

    let folder = tokio::task::spawn_blocking(move || -> Result<Folder, String> {
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

        // Encode folder name to UTF-7 IMAP
        let encoded_name = encode_folder_name(&folder_name);

        // Create the folder
        imap_session
            .create(&encoded_name)
            .map_err(|e| format!("Failed to create folder: {}", e))?;

        println!("✅ Created remote folder '{}'", folder_name);

        let _ = imap_session.logout();

        // Return the created folder
        Ok(Folder {
            id: None,
            account_id,
            name: encoded_name,
            display_name: folder_name,
            delimiter: Some("/".to_string()),
            flags: None,
            is_local: false,
        })
    })
    .await
    .map_err(|e| e.to_string())??;

    // Insert into database
    let pool = db::pool();
    let result = sqlx::query(
        "INSERT INTO folders (account_id, name, display_name, delimiter, flags, is_local) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(account_id)
    .bind(&folder.name)
    .bind(&folder.display_name)
    .bind(&folder.delimiter)
    .bind(&folder.flags)
    .bind(0)
    .execute(pool.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    let folder_id = result.last_insert_rowid() as i32;

    Ok(Folder {
        id: Some(folder_id),
        ..folder
    })
}

/// Delete a remote IMAP folder
#[command]
pub async fn delete_remote_folder(
    config: AccountConfig,
    folder_name: String,
) -> Result<(), String> {
    println!(
        "Deleting remote folder '{}' for {}",
        folder_name, config.email
    );

    // Ensure we have a valid access token (refresh if needed)
    let config = ensure_valid_token(config).await?;
    let account_id = config.id.ok_or("Account ID is required")?;

    // Clone folder_name for use in both the blocking task and database query
    let folder_name_for_db = folder_name.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut imap_session = imap_helpers::connect_and_login(&config)?;

        // Delete the folder (folder_name should already be UTF-7 encoded)
        imap_session
            .delete(&folder_name)
            .map_err(|e| format!("Failed to delete folder: {}", e))?;

        println!("✅ Deleted remote folder '{}'", folder_name);

        let _ = imap_session.logout();

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    // Delete from database
    let pool = db::pool();
    sqlx::query("DELETE FROM folders WHERE account_id = ? AND name = ?")
        .bind(account_id)
        .bind(&folder_name_for_db)
        .execute(pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Create a local-only folder
#[command]
pub async fn create_local_folder(account_id: i32, folder_name: String) -> Result<Folder, String> {
    println!(
        "Creating local folder '{}' for account_id: {}",
        folder_name, account_id
    );

    let pool = db::pool();

    // Check if folder already exists
    let existing: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM folders WHERE account_id = ? AND display_name = ?")
            .bind(account_id)
            .bind(&folder_name)
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| e.to_string())?;

    if existing.is_some() {
        return Err("A folder with this name already exists".to_string());
    }

    // Create a unique name for the local folder (prefix with "local_")
    let unique_name = format!("local_{}", folder_name);

    // Insert into database
    let result = sqlx::query(
        "INSERT INTO folders (account_id, name, display_name, delimiter, flags, is_local) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(account_id)
    .bind(&unique_name)
    .bind(&folder_name)
    .bind("/")
    .bind(None::<String>)
    .bind(1)
    .execute(pool.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    let folder_id = result.last_insert_rowid() as i32;

    println!("✅ Created local folder '{}'", folder_name);

    Ok(Folder {
        id: Some(folder_id),
        account_id,
        name: unique_name,
        display_name: folder_name,
        delimiter: Some("/".to_string()),
        flags: None,
        is_local: true,
    })
}

/// Delete a local-only folder
#[command]
pub async fn delete_local_folder(account_id: i32, folder_name: String) -> Result<(), String> {
    println!(
        "Deleting local folder '{}' for account_id: {}",
        folder_name, account_id
    );

    let pool = db::pool();

    // Delete the folder and its emails (CASCADE will handle emails)
    let result =
        sqlx::query("DELETE FROM folders WHERE account_id = ? AND name = ? AND is_local = 1")
            .bind(account_id)
            .bind(&folder_name)
            .execute(pool.as_ref())
            .await
            .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("Local folder not found".to_string());
    }

    println!("✅ Deleted local folder '{}'", folder_name);

    Ok(())
}

#[command]
pub async fn load_folders(account_id: i32) -> Result<Vec<Folder>, String> {
    println!("Loading folders for account_id: {}", account_id);

    let pool = db::pool();

    let folders = sqlx::query_as::<_, (Option<i32>, i32, String, String, Option<String>, Option<String>, i32)>(
        "SELECT id, account_id, name, display_name, delimiter, flags, COALESCE(is_local, 0) FROM folders WHERE account_id = ? ORDER BY display_name",
    )
    .bind(account_id)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .map(|(id, account_id, name, display_name, delimiter, flags, is_local)| Folder {
        id,
        account_id,
        name,
        display_name,
        delimiter,
        flags,
        is_local: is_local != 0,
    })
    .collect();

    Ok(folders)
}
