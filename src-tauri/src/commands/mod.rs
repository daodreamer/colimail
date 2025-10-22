pub mod accounts;
pub mod emails;
pub mod folders;
pub mod oauth2;
pub mod send;
pub mod utils;

pub use accounts::{delete_account, load_account_configs, save_account_config};
pub use emails::{delete_email, fetch_email_body, fetch_emails, move_email_to_trash};
pub use folders::{fetch_folders, load_folders, sync_folders};
pub use oauth2::{complete_oauth2_flow, listen_for_oauth_callback, start_oauth2_flow};
pub use send::{forward_email, reply_email, send_email};
