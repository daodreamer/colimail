pub mod accounts;
pub mod drafts;
pub mod emails;
pub mod folders;
pub mod notifications;
pub mod oauth2;
pub mod send;
pub mod test_connection;
pub mod utils; // Public so idle_manager can access ensure_valid_token

pub use accounts::{delete_account, load_account_configs, save_account_config};
pub use drafts::{delete_draft, list_drafts, load_draft, save_draft};
pub use emails::{
    delete_email, download_attachment, fetch_email_body, fetch_email_body_cached, fetch_emails,
    get_last_sync_time, get_sync_interval, load_attachments_info, load_emails_from_cache,
    mark_email_as_flagged, mark_email_as_read, mark_email_as_unflagged, mark_email_as_unread,
    move_email_to_trash, save_attachment_to_file, set_sync_interval, should_sync, sync_email_flags,
    sync_emails, sync_specific_email_flags,
};
pub use folders::{fetch_folders, load_folders, sync_folders};
pub use notifications::{
    get_notification_enabled, get_sound_enabled, set_notification_enabled, set_sound_enabled,
};
pub use oauth2::{complete_oauth2_flow, listen_for_oauth_callback, start_oauth2_flow};
pub use send::{forward_email, get_attachment_size_limit, reply_email, send_email};
pub use test_connection::test_connection;
