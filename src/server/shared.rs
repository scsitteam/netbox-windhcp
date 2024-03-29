use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::Mutex;

#[derive(Debug, Default, Serialize)]
pub enum SyncStatus {
    #[default]
    Unknown,
    SyncOk,
    SyncFailed,
}

#[derive(Debug, Default, Serialize)]
pub struct ServerStatus {
    pub needs_sync: bool,
    pub syncing: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_sync_status: SyncStatus,
}

impl ServerStatus {
    pub fn new() -> Self { Self { ..Default::default() } }
}

pub type SharedServerStatus = Arc<Mutex<ServerStatus>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Shutdown,
    TriggerSync,
}
