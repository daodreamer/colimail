// IDLE manager module
// This module manages IMAP IDLE connections for real-time email notifications

// Sub-modules
mod manager;
mod notification;
mod session;
mod types;

// Re-export public types and manager
pub use manager::IdleManager;
pub use types::IdleCommand;
