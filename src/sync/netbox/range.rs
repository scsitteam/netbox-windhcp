use std::net::Ipv4Addr;

use ipnet::Ipv4Net;
use serde::Deserialize;

use super::prefix::Prefix;

#[derive(Debug, Deserialize)]
pub struct IpRange {
    start_address: Ipv4Net,
    end_address: Ipv4Net,
}

impl IpRange {
    pub fn start_address(&self) -> Ipv4Addr {
        self.start_address.addr()
    }

    pub fn end_address(&self) -> Ipv4Addr {
        self.end_address.addr()
    }
    
    pub fn is_contained(&self, prefix: &Prefix) -> bool {
        prefix.prefix().contains(&self.start_address) && prefix.prefix().contains(&self.end_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_netbox_ip_range() {
        let range = serde_json::from_str::<IpRange>(r#"{
            "start_address": "192.168.1.100/22",
            "end_address": "192.168.1.149/22",
            "description": "Future use"
        }"#);
        assert!(range.is_ok());
        let range = range.unwrap();

        assert_eq!(range.start_address(), "192.168.1.100".parse::<Ipv4Addr>().unwrap());
        assert_eq!(range.end_address(), "192.168.1.149".parse::<Ipv4Addr>().unwrap());
    }

    #[test]
    fn returns_true_if_it_is_contained() {
        let prefix = serde_json::from_str::<Prefix>(r#"{
            "prefix": "10.112.130.0/24",
            "description": "foo",
            "custom_fields": {}
        }"#).unwrap();
        let range = serde_json::from_str::<IpRange>(r#"{
            "start_address": "10.112.130.100/24",
            "end_address": "10.112.130.149/24"
        }"#).unwrap();

        assert!(range.is_contained(&prefix));
    }

    #[test]
    fn returns_false_if_it_is_not_contained() {
        let prefix = serde_json::from_str::<Prefix>(r#"{
            "prefix": "10.112.130.0/24",
            "description": "foo",
            "custom_fields": {}
        }"#).unwrap();
        let range = serde_json::from_str::<IpRange>(r#"{
            "start_address": "192.168.1.100/22",
            "end_address": "192.168.1.149/22"
        }"#).unwrap();

        assert!(!range.is_contained(&prefix));
    }
}