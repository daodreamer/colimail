// IDLE manager data types and global state
// This module defines the core data structures used throughout the IDLE manager

use crate::models::AccountConfig;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

/// Notification data to be queued
#[derive(Debug, Clone)]
pub struct NotificationData {
    pub title: String,
    pub from: String,
    pub subject: String,
}

/// Global notification queue
pub static NOTIFICATION_QUEUE: Lazy<Arc<Mutex<Vec<NotificationData>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

/// Flag to track if notification worker is running
pub static NOTIFICATION_WORKER_RUNNING: Lazy<Arc<Mutex<bool>>> =
    Lazy::new(|| Arc::new(Mutex::new(false)));

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
