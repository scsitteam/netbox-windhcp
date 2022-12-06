use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SyncConfig {
    pub netbox: SyncNetboxConfig,
    pub dhcp: SyncDhcpConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncNetboxConfig {
    pub apiurl: String,
    pub token: String,
    pub prefix_filter: HashMap<String, String>,
    pub range_filter: HashMap<String, String>,
    pub reservation_filter: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncDhcpConfig {
    pub server: String,
    pub lease_duration: Option<u32>,
}