use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
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
    #[serde(default)]
    pub flagged: bool, // Starred/flagged status
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
    pub name: String, // Original IMAP folder name in UTF-7 encoding (for IMAP operations)
    pub display_name: String, // User-friendly display name (UTF-7 decoded)
    pub delimiter: Option<String>,
    pub flags: Option<String>,
    #[serde(default)]
    pub is_local: bool, // True for local-only folders, False for remote IMAP folders
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

#[cfg(test)]
mod folder_tests {
    use super::*;

    #[test]
    fn test_is_selectable_without_flags() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "INBOX".to_string(),
            display_name: "Inbox".to_string(),
            delimiter: Some("/".to_string()),
            flags: None,
            is_local: false,
        };
        assert!(folder.is_selectable());
    }

    #[test]
    fn test_is_selectable_with_noselect_flag() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "[Gmail]".to_string(),
            display_name: "[Gmail]".to_string(),
            delimiter: Some("/".to_string()),
            flags: Some("\\Noselect".to_string()),
            is_local: false,
        };
        assert!(!folder.is_selectable());
    }

    #[test]
    fn test_is_selectable_case_insensitive() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "[Gmail]".to_string(),
            display_name: "[Gmail]".to_string(),
            delimiter: Some("/".to_string()),
            flags: Some("\\NOSELECT".to_string()),
            is_local: false,
        };
        assert!(!folder.is_selectable());
    }

    #[test]
    fn test_is_selectable_with_other_flags() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "INBOX".to_string(),
            display_name: "Inbox".to_string(),
            delimiter: Some("/".to_string()),
            flags: Some("\\HasChildren".to_string()),
            is_local: false,
        };
        assert!(folder.is_selectable());
    }

    #[test]
    fn test_should_show_to_user_normal_folder() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "INBOX".to_string(),
            display_name: "Inbox".to_string(),
            delimiter: Some("/".to_string()),
            flags: None,
            is_local: false,
        };
        assert!(folder.should_show_to_user());
    }

    #[test]
    fn test_should_show_to_user_sync_issues_chinese() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "同步问题".to_string(),
            display_name: "同步问题".to_string(),
            delimiter: Some("/".to_string()),
            flags: None,
            is_local: false,
        };
        assert!(!folder.should_show_to_user());
    }

    #[test]
    fn test_should_show_to_user_sync_issues_english() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "Sync Issues".to_string(),
            display_name: "Sync Issues".to_string(),
            delimiter: Some("/".to_string()),
            flags: None,
            is_local: false,
        };
        assert!(!folder.should_show_to_user());
    }

    #[test]
    fn test_should_show_to_user_subfolder_of_system_folder() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "Sync Issues/Server Failures".to_string(),
            display_name: "Sync Issues/Server Failures".to_string(),
            delimiter: Some("/".to_string()),
            flags: None,
            is_local: false,
        };
        assert!(!folder.should_show_to_user());
    }

    #[test]
    fn test_should_show_to_user_noselect_folder() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "[Gmail]".to_string(),
            display_name: "[Gmail]".to_string(),
            delimiter: Some("/".to_string()),
            flags: Some("\\Noselect".to_string()),
            is_local: false,
        };
        assert!(!folder.should_show_to_user());
    }

    #[test]
    fn test_should_show_to_user_rss_feeds() {
        let folder = Folder {
            id: Some(1),
            account_id: 1,
            name: "RSS Feeds".to_string(),
            display_name: "RSS Feeds".to_string(),
            delimiter: Some("/".to_string()),
            flags: None,
            is_local: false,
        };
        assert!(!folder.should_show_to_user());
    }
}

#[cfg(test)]
mod serde_tests {
    use super::*;

    #[test]
    fn test_auth_type_serde() {
        // Test Basic auth type serialization
        let basic = AuthType::Basic;
        let json = serde_json::to_string(&basic).unwrap();
        assert_eq!(json, "\"basic\"");

        // Test OAuth2 auth type serialization
        let oauth = AuthType::OAuth2;
        let json = serde_json::to_string(&oauth).unwrap();
        assert_eq!(json, "\"oauth2\"");

        // Test deserialization
        let auth: AuthType = serde_json::from_str("\"basic\"").unwrap();
        assert_eq!(auth, AuthType::Basic);

        let auth: AuthType = serde_json::from_str("\"oauth2\"").unwrap();
        assert_eq!(auth, AuthType::OAuth2);
    }

    #[test]
    fn test_account_config_serde_basic() {
        let account = AccountConfig {
            id: Some(1),
            email: "test@example.com".to_string(),
            password: Some("password123".to_string()),
            imap_server: "imap.example.com".to_string(),
            imap_port: 993,
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            auth_type: Some(AuthType::Basic),
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            display_name: Some("Test User".to_string()),
        };

        // Serialize
        let json = serde_json::to_string(&account).unwrap();
        assert!(json.contains("\"email\":\"test@example.com\""));
        assert!(json.contains("\"auth_type\":\"basic\""));

        // Deserialize
        let deserialized: AccountConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.email, "test@example.com");
        assert_eq!(deserialized.imap_port, 993);
    }

    #[test]
    fn test_account_config_skip_none_fields() {
        let account = AccountConfig {
            id: Some(1),
            email: "test@example.com".to_string(),
            password: None, // Should be skipped in serialization
            imap_server: "imap.example.com".to_string(),
            imap_port: 993,
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            auth_type: Some(AuthType::OAuth2),
            access_token: None,     // Should be skipped
            refresh_token: None,    // Should be skipped
            token_expires_at: None, // Should be skipped
            display_name: None,     // Should be skipped
        };

        let json = serde_json::to_string(&account).unwrap();
        // None fields should not appear in JSON
        assert!(!json.contains("\"password\""));
        assert!(!json.contains("\"access_token\""));
        assert!(!json.contains("\"refresh_token\""));
        assert!(!json.contains("\"token_expires_at\""));
        assert!(!json.contains("\"display_name\""));
    }

    #[test]
    fn test_email_header_default_fields() {
        let json = r#"{
            "uid": 123,
            "subject": "Test",
            "from": "sender@example.com",
            "to": "receiver@example.com",
            "date": "Mon, 15 Jan 2024 14:30:00 +0000",
            "timestamp": 1705329000
        }"#;

        let email: EmailHeader = serde_json::from_str(json).unwrap();
        assert_eq!(email.uid, 123);
        assert_eq!(email.cc, ""); // Default value
        assert!(!email.has_attachments); // Default value
        assert!(!email.seen); // Default value
        assert!(!email.flagged); // Default value
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DraftType {
    Compose,
    Reply,
    Forward,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DraftListItem {
    pub id: i64,
    pub account_id: i32,
    pub to_addr: String,
    pub cc_addr: String,
    pub subject: String,
    pub draft_type: DraftType,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RewardStatus {
    Available,
    Claimed,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RewardCacheItem {
    pub reward_id: String,
    pub email_hash: String,
    pub amount: String,
    pub sender: String,
    pub recipient: String,
    pub status: RewardStatus,
    pub timestamp: i64,
    pub updated_at: i64,
}
