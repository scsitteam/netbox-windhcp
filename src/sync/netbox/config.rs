use std::collections::HashMap;

use ipnet::Ipv4Net;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct SyncNetboxConfig {
    apiurl: String,
    token: String,
    prefix_filter: HashMap<String, String>,
    range_filter: HashMap<String, String>,
    reservation_filter: HashMap<String, String>,
    router_filter: HashMap<String, String>,
}

impl Default for SyncNetboxConfig {
    fn default() -> Self {
        Self {
            apiurl: Default::default(),
            token: Default::default(),
            prefix_filter: HashMap::from([
                (String::from("tag"), String::from("dhcp")),
                (String::from("status"), String::from("active")),
                (String::from("family"), String::from("4")),
            ]),
            range_filter: HashMap::from([
                (String::from("role"), String::from("dhcp-pool")),
                (String::from("status"), String::from("active")),
                (String::from("family"), String::from("4")),
            ]),
            reservation_filter: HashMap::from([
                (String::from("tag"), String::from("dhcp")),
                (String::from("status"), String::from("active")),
            ]),
            router_filter: HashMap::from([
                (String::from("tag"), String::from("dhcp")),
                (String::from("status"), String::from("active")),
            ]),
        }
    }
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

    pub fn reservation_filter(&self, parent: &Ipv4Net) -> HashMap<String, String> {
        let mut filter = self.reservation_filter.clone();
        filter.insert(String::from("parent"), parent.to_string());
        filter
    }

    pub fn router_filter(&self, parent: &Ipv4Net) -> HashMap<String, String> {
        let mut filter = self.router_filter.clone();
        filter.insert(String::from("parent"), parent.to_string());
        filter
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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
        router_filter:
            ip: addr
        "#);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();
        assert_eq!(cfg.apiurl, "https://netbox.example.com/api/");
        assert_eq!(cfg.token, "SECRET");
        assert_eq!(cfg.prefix_filter.get("foo").unwrap(), "bar");
        assert_eq!(cfg.range_filter.get("alice").unwrap(), "bob");
        assert_eq!(cfg.reservation_filter.get("tag").unwrap(), "value");
        assert_eq!(cfg.router_filter.get("ip").unwrap(), "addr");
    }

    #[test]
    fn it_parses_minimal_netbox_config() {
        let cfg = serde_yaml::from_str::<SyncNetboxConfig>(r#"---
        apiurl: https://netbox.example.com/api/
        token: SECRET
        "#);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();
        assert_eq!(cfg.apiurl, "https://netbox.example.com/api/");
        assert_eq!(cfg.token, "SECRET");
        assert_eq!(cfg.prefix_filter.get("tag").unwrap(), "dhcp");
        assert_eq!(cfg.prefix_filter.get("status").unwrap(), "active");
    }

    #[test]
    fn it_builds_the_reservation_filter() {
        let cfg = SyncNetboxConfig::default();
        let filter = cfg.reservation_filter(&Ipv4Net::from_str("127.0.0.1/8").unwrap());
        assert_eq!(filter.get("parent").unwrap(), "127.0.0.1/8");
    }

    #[test]
    fn it_builds_the_router_filter() {
        let cfg = SyncNetboxConfig::default();
        let filter = cfg.router_filter(&Ipv4Net::from_str("127.0.0.1/8").unwrap());
        assert_eq!(filter.get("parent").unwrap(), "127.0.0.1/8");
    }
}
