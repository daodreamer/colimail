// Email fetching operations from IMAP server
// This module handles retrieving email headers and bodies
//
// Organized by responsibility:
// - list.rs: Fetch email list/headers
// - body.rs: Fetch email body and attachments
// - headers.rs: Fetch raw headers (for CMVH verification)

// Sub-modules organized by responsibility
mod body;
mod headers;
mod list;

// Re-export public command functions for use in commands/emails/mod.rs
pub use body::{fetch_email_body, fetch_email_body_cached};
pub use headers::fetch_email_raw_headers;
pub use list::fetch_emails;

/// OAuth2 authenticator for IMAP
/// This struct is used by imap_helpers::connect_and_login for OAuth2 authentication
pub struct OAuth2 {
    pub user: String,
    pub access_token: String,
}

impl imap::Authenticator for OAuth2 {
    type Response = String;
    fn process(&self, _: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}
