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

            // Decode from UTF-7 IMAP encoding
            let decoded_name = decode_folder_name(&raw_name);

            // Generate user-friendly display name
            let display_name = get_display_name(&decoded_name);

            let delimiter = mailbox.delimiter().map(|d| d.to_string());
            let flags = Some(format!("{:?}", mailbox.attributes()));

            let folder = Folder {
                id: None,
                account_id,
                name: decoded_name.clone(), // Store decoded name for IMAP operations
                display_name: display_name.clone(),
                delimiter,
                flags: flags.clone(),
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
                let folder_name_for_log = folder.name.clone();
                match imap_session.select(&folder.name) {
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
        sqlx::query("INSERT INTO folders (account_id, name, display_name, delimiter, flags) VALUES (?, ?, ?, ?, ?)")
            .bind(account_id)
            .bind(&folder.name)
            .bind(&folder.display_name)
            .bind(&folder.delimiter)
            .bind(&folder.flags)
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

#[command]
pub async fn load_folders(account_id: i32) -> Result<Vec<Folder>, String> {
    println!("Loading folders for account_id: {}", account_id);

    let pool = db::pool();

    let folders = sqlx::query_as::<_, (Option<i32>, i32, String, String, Option<String>, Option<String>)>(
        "SELECT id, account_id, name, display_name, delimiter, flags FROM folders WHERE account_id = ? ORDER BY display_name",
    )
    .bind(account_id)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .map(|(id, account_id, name, display_name, delimiter, flags)| Folder {
        id,
        account_id,
        name,
        display_name,
        delimiter,
        flags,
    })
    .collect();

    Ok(folders)
}
