use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SyncConfig {
    pub netbox: SyncNetboxConfig,
    pub dhcp: SyncDhcpConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncNetboxConfig {
    apiurl: String,
    token: String,
    prefix_filter: HashMap<String, String>,
    range_filter: HashMap<String, String>,
    reservation_filter: HashMap<String, String>,
}

impl SyncNetboxConfig {
    pub fn apiurl(&self) -> &str {
        self.apiurl.as_ref()
    }

    pub fn token(&self) -> &str {
        self.token.as_ref()
    }

    pub fn prefix_filter(&self) -> &HashMap<String, String> {
        &self.prefix_filter
    }

    pub fn range_filter(&self) -> &HashMap<String, String> {
        &self.range_filter
    }

    pub fn reservation_filter(&self) -> &HashMap<String, String> {
        &self.reservation_filter
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncDhcpConfig {
    server: String,
    lease_duration: Option<u32>,
}

impl SyncDhcpConfig {
    pub fn server(&self) -> &str {
        self.server.as_ref()
    }

    pub fn lease_duration(&self) -> u32 {
        self.lease_duration.unwrap_or(7 * 24 * 60 * 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_netbox_config() {
        let cfg = serde_yaml::from_str::<SyncNetboxConfig>(r#"---
        apiurl: https://netbox.example.com/api/
        token: SECRET
        prefix_filter:
            foo: bar
        range_filter:
            alice: bob
        reservation_filter:
            tag: value
        "#);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();
        assert_eq!(cfg.apiurl(), "https://netbox.example.com/api/");
        assert_eq!(cfg.token(), "SECRET");
        assert_eq!(cfg.prefix_filter().get("foo").unwrap(), "bar");
        assert_eq!(cfg.range_filter().get("alice").unwrap(), "bob");
        assert_eq!(cfg.reservation_filter().get("tag").unwrap(), "value");
    }

    #[test]
    fn it_parses_dhcp_config() {
        let cfg = serde_yaml::from_str::<SyncDhcpConfig>(r#"---
        server: dhcp.example.com
        lease_duration: 3600
        "#);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();
        assert_eq!(cfg.server(), "dhcp.example.com");
        assert_eq!(cfg.lease_duration(), 3600);
    }

    #[test]
    fn it_parses_dhcp_config_without_lease_duration() {
        let cfg = serde_yaml::from_str::<SyncDhcpConfig>(r#"---
        server: dhcp.example.com
        "#);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();
        assert_eq!(cfg.server(), "dhcp.example.com");
        assert_eq!(cfg.lease_duration(), 604800);
    }
}