// Email commands module
// This module handles all email-related IMAP operations

pub mod attachments;
pub mod cache;
pub mod codec;
pub mod delete;
pub mod fetch;
pub mod flags;
pub mod imap_helpers;
pub mod sync;
pub mod sync_interval;

// Re-export public command functions for use in main.rs
pub use attachments::{download_attachment, load_attachments_info, save_attachment_to_file};
pub use cache::load_emails_from_cache;
pub use delete::{delete_email, move_email_to_trash};
pub use fetch::{fetch_email_body, fetch_email_body_cached, fetch_email_raw_headers, fetch_emails};
pub use flags::{
    mark_email_as_flagged, mark_email_as_read, mark_email_as_unflagged, mark_email_as_unread,
};
pub use sync::{
    get_last_sync_time, should_sync, sync_email_flags, sync_emails, sync_specific_email_flags,
};
pub use sync_interval::{get_sync_interval, set_sync_interval};
