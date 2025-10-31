// IDLE manager data types and global state
// This module defines the core data structures used throughout the IDLE manager

use crate::models::AccountConfig;

/// Command to control the IDLE manager
#[derive(Debug)]
pub enum IdleCommand {
    Start {
        account_id: i32,
        folder_name: String,
        config: AccountConfig,
    },
    Stop {
        account_id: i32,
        folder_name: String,
    },
    StopAll,
    StartAllForAccount {
        config: AccountConfig,
    },
    StopAllForAccount {
        account_id: i32,
    },
}

/// Event emitted by IDLE connections
#[derive(Debug, Clone, serde::Serialize)]
pub struct IdleEvent {
    pub account_id: i32,
    pub folder_name: String,
    pub event_type: IdleEventType,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum IdleEventType {
    NewMessages { count: u32 },
    Expunge { uid: u32 },
    FlagsChanged { uid: u32 },
    ConnectionLost,
}
