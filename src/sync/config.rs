use std::{collections::HashMap, net::Ipv4Addr};

use serde::Deserialize;

use super::windhcp::DnsFlags;

#[derive(Debug, Deserialize, Clone)]
pub struct SyncConfig {
    pub netbox: SyncNetboxConfig,
    pub dhcp: SyncDhcpConfig,
}

impl SyncConfig {
    pub fn netbox(&self) -> &SyncNetboxConfig {
        &self.netbox
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncNetboxConfig {
    pub(super) apiurl: String,
    pub(super) token: String,
    pub(super) prefix_filter: HashMap<String, String>,
    pub(super) range_filter: HashMap<String, String>,
    pub(super) reservation_filter: HashMap<String, String>,
    pub(super) router_filter: HashMap<String, String>,
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

    pub fn router_filter(&self) -> &HashMap<String, String> {
        &self.router_filter
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncDhcpConfig {
    server: String,
    lease_duration: Option<u32>,
    #[serde(default)]
    default_dns_flags: DnsFlags,
    default_dns_domain: Option<String>,
    #[serde(default)]
    default_dns_servers: Vec<Ipv4Addr>,
}

impl SyncDhcpConfig {
    pub fn server(&self) -> &str {
        self.server.as_ref()
    }

    pub fn lease_duration(&self) -> u32 {
        self.lease_duration.unwrap_or(7 * 24 * 60 * 60)
    }

    pub fn default_dns_flags(&self) -> &DnsFlags {
        &self.default_dns_flags
    }

    pub fn default_dns_domain(&self) -> Option<&String> {
        self.default_dns_domain.as_ref()
    }

    pub fn default_dns_servers(&self) -> &[Ipv4Addr] {
        self.default_dns_servers.as_ref()
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
        default_dns_flags:
            enabled: true
            cleanup_expired: true
            update_dhcid: true
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
        dbg!(&cfg);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();
        assert_eq!(cfg.server(), "dhcp.example.com");
        assert_eq!(cfg.lease_duration(), 604800);
    }
}