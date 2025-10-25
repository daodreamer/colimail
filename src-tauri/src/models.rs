use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Basic,
    OAuth2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountConfig {
    pub id: Option<i32>,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
    #[serde(default)]
    pub auth_type: Option<AuthType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OAuth2StartRequest {
    pub provider: String, // "google" or "outlook"
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OAuth2StartResponse {
    pub auth_url: String,
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailHeader {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub cc: String, // CC recipients
    pub date: String,
    pub timestamp: i64, // Unix timestamp in seconds for sorting and conversion
    #[serde(default)]
    pub has_attachments: bool, // Indicates if email has attachments
    #[serde(default)]
    pub seen: bool, // Read/unread status
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attachment {
    pub id: Option<i64>,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>, // Optional: only included when downloading
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttachmentInfo {
    pub id: i64,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    pub id: Option<i32>,
    pub account_id: i32,
    pub name: String,         // Original IMAP folder name (for IMAP operations)
    pub display_name: String, // User-friendly display name
    pub delimiter: Option<String>,
    pub flags: Option<String>,
}

impl Folder {
    /// Check if the folder is selectable (not marked with \Noselect flag)
    pub fn is_selectable(&self) -> bool {
        if let Some(ref flags) = self.flags {
            // Check if flags contain "Noselect" (case-insensitive)
            !flags.to_lowercase().contains("noselect")
        } else {
            true // If no flags, assume it's selectable
        }
    }

    /// Check if this folder should be shown to the user
    /// Filters out system folders that users typically don't need to access
    pub fn should_show_to_user(&self) -> bool {
        // First check if it's selectable
        if !self.is_selectable() {
            return false;
        }

        // Filter out known system/troubleshooting folders by display name
        let system_folder_patterns = [
            "同步问题",                      // Outlook sync issues (Chinese)
            "Sync Issues",                   // Outlook sync issues (English)
            "Recoverable Items",             // Outlook recoverable items
            "Conversation History",          // Skype/Teams conversation history
            "RSS Feeds",                     // RSS subscriptions
            "Social Activity Notifications", // Social updates
            "Suggested Contacts",            // Auto-discovered contacts
        ];

        let lower_display_name = self.display_name.to_lowercase();

        // Check if display name starts with any system folder pattern
        for pattern in &system_folder_patterns {
            if lower_display_name.starts_with(&pattern.to_lowercase()) {
                return false;
            }
        }

        // Check if it's a subfolder of a filtered folder
        // (e.g., "同步问题/本地故障" should also be filtered)
        for pattern in &system_folder_patterns {
            let pattern_lower = pattern.to_lowercase();
            if lower_display_name.contains(&format!("{}/", pattern_lower))
                || lower_display_name.contains(&pattern_lower)
            {
                return false;
            }
        }

        true
    }
}
