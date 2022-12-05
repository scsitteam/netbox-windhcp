use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NetboxWebHook {
    pub event: NetboxWebHookEvent,
    pub timestamp: DateTime<Utc>,
    pub model: NetboxWebHookModel,
    pub username: String,
    pub request_id: String,
    pub data: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum NetboxWebHookEvent {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum NetboxWebHookModel {
    Prefix,
    Updated,
    Deleted,
}