use std::net::Ipv4Addr;
use std::path::PathBuf;

use serde::Deserialize;

use super::netbox::config::SyncNetboxConfig;

use super::windhcp::DnsFlags;

#[derive(Debug, Deserialize, Clone)]
pub struct SyncConfig {
    pub netbox: SyncNetboxConfig,
    pub dhcp: SyncDhcpConfig,
    pub logs: SyncLogConfig,
}

impl SyncConfig {
    pub fn netbox(&self) -> &SyncNetboxConfig {
        &self.netbox
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
    default_failover_relation: Option<String>,
}

impl SyncDhcpConfig {
    pub fn server(&self) -> &str {
        self.server.as_ref()
    }

    pub fn lease_duration(&self) -> u32 {
        self.lease_duration.unwrap_or(7 * 24 * 60 * 60)
    }

    pub fn default_dns_flags(&self) -> Option<DnsFlags> {
        Some(self.default_dns_flags.clone())
    }

    pub fn default_dns_domain(&self) -> Option<&String> {
        self.default_dns_domain.as_ref()
    }

    pub fn default_dns_servers(&self) -> &[Ipv4Addr] {
        self.default_dns_servers.as_ref()
    }

    pub fn default_failover_relation(&self) -> Option<&String> {
        self.default_failover_relation.as_ref()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncLogConfig {
    pub dir: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
