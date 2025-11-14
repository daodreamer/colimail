pub mod accounts;
pub mod auth;
pub mod cmvh;
pub mod detect_display_name;
pub mod drafts;
pub mod emails;
pub mod encryption_manager;
pub mod folders;
pub mod logs;
pub mod notifications;
pub mod oauth2;
pub mod send;
pub mod send_cmvh;
pub mod test_connection;
pub mod utils; // Public so idle_manager can access ensure_valid_token

pub use accounts::{delete_account, load_account_configs, save_account_config};
pub use auth::{
    delete_app_user, delete_secure_storage, get_app_user, get_secure_storage, set_secure_storage,
    sync_app_user,
};
pub use cmvh::{
    cleanup_cmvh_cache, clear_cmvh_cache, derive_eth_address, get_cmvh_cache, get_cmvh_cache_stats,
    has_cmvh_headers, hash_email_content, parse_email_cmvh_headers, save_cmvh_cache,
    sign_email_with_cmvh, verify_cmvh_signature,
};
pub use detect_display_name::detect_display_name_from_sent;
pub use drafts::{delete_draft, list_drafts, load_draft, save_draft};
pub use emails::{
    delete_email, download_attachment, fetch_email_body, fetch_email_body_cached,
    fetch_email_raw_headers, fetch_emails, get_last_sync_time, get_sync_interval,
    load_attachments_info, load_emails_from_cache, mark_email_as_flagged, mark_email_as_read,
    mark_email_as_unflagged, mark_email_as_unread, move_email_to_trash, save_attachment_to_file,
    set_sync_interval, should_sync, sync_email_flags, sync_emails, sync_specific_email_flags,
};
pub use encryption_manager::{
    change_master_password, disable_encryption, enable_encryption, get_encryption_status,
    lock_encryption_command, unlock_encryption_with_password,
};
pub use folders::{
    check_folder_capabilities, create_local_folder, create_remote_folder, delete_local_folder,
    delete_remote_folder, fetch_folders, load_folders, sync_folders,
};
pub use logs::{
    export_logs_as_zip, get_current_log_file, get_log_directory, list_log_files, read_log_file,
    read_recent_logs,
};
pub use notifications::{
    get_minimize_to_tray, get_notification_enabled, get_sound_enabled, set_minimize_to_tray,
    set_notification_enabled, set_sound_enabled,
};
pub use oauth2::{complete_oauth2_flow, listen_for_oauth_callback, start_oauth2_flow};
pub use send::{forward_email, get_attachment_size_limit, reply_email, send_email};
pub use send_cmvh::{send_email_smtp, send_email_with_cmvh, sign_email_cmvh};
pub use test_connection::test_connection;
