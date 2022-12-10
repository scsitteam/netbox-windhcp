use std::net::Ipv4Addr;

use ipnet::Ipv4Net;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Prefix {
    prefix: Ipv4Net,
    description: String,
    custom_fields: PrefixCustomField,
}

impl Prefix {
    pub fn prefix(&self) -> Ipv4Net {
        self.prefix
    }

    pub fn addr(&self) -> Ipv4Addr {
        self.prefix.addr()
    }

    pub fn netmask(&self) -> Ipv4Addr {
        self.prefix.netmask()
    }

    pub fn description(&self) -> &str {
        self.description.as_ref()
    }

    pub fn lease_duration(&self) -> Option<u32> {
        self.custom_fields.dhcp_lease_duration
    }

    pub fn dns_flags(&self) -> Option<&Vec<String>> {
        self.custom_fields.dhcp_dns_flags.as_ref()
    }

    pub fn routers(&self) -> Option<Vec<Ipv4Addr>> {
        self.custom_fields.dhcp_routers.as_ref()
            .map(|routers| routers.iter().map(|n| n.address.addr())
            .collect::<Vec<Ipv4Addr>>())
    }

    pub fn dns_domain(&self) -> Option<&String> {
        self.custom_fields.dhcp_dns_domain.as_ref()
    }

    pub fn dns_servers(&self) -> Option<Vec<Ipv4Addr>> {
        self.custom_fields.dhcp_dns_servers.as_ref()
            .map(|dns| dns.iter().map(|n| n.address.addr())
            .collect::<Vec<Ipv4Addr>>())
    }
}

#[derive(Debug, Deserialize)]
struct PrefixCustomField {
    dhcp_lease_duration: Option<u32>,
    dhcp_dns_flags: Option<Vec<String>>,
    dhcp_routers: Option<Vec<PrefixCustomFieldIp>>,
    dhcp_dns_domain: Option<String>,
    dhcp_dns_servers: Option<Vec<PrefixCustomFieldIp>>,
}

#[derive(Debug, Deserialize)]
struct PrefixCustomFieldIp {
    address: Ipv4Net,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_netbox_prefix() {
        let prefix = serde_json::from_str::<Prefix>(r#"{
            "prefix": "10.112.130.0/24",
            "description": "foo",
            "custom_fields": {}
        }"#);
        assert!(prefix.is_ok());
        let prefix = prefix.unwrap();

        assert_eq!(prefix.addr(), "10.112.130.0".parse::<Ipv4Addr>().unwrap());
        assert_eq!(prefix.netmask(), "255.255.255.0".parse::<Ipv4Addr>().unwrap());
        assert_eq!(prefix.description(), "foo");
    }

    #[test]
    fn it_parses_netbox_prefix_w_cf() {
        let prefix = serde_json::from_str::<Prefix>(r#"{
            "prefix": "10.112.130.0/24",
            "description": "foo",
            "custom_fields": {
                "dhcp_lease_duration": 86400,
                "dhcp_dns_flags": ["enabled"],
                "dhcp_routers": [
                    { "address": "10.112.130.1/24" }
                ],
                "dhcp_dns_domain": "example.com",
                "dhcp_dns_servers": [
                    { "address": "10.112.130.2/24" },
                    { "address": "10.112.130.3/24" }
                ]
            }
        }"#);
        dbg!(&prefix);
        assert!(prefix.is_ok());
        let prefix = prefix.unwrap();

        assert_eq!(prefix.addr(), "10.112.130.0".parse::<Ipv4Addr>().unwrap());
        assert_eq!(prefix.netmask(), "255.255.255.0".parse::<Ipv4Addr>().unwrap());
        assert_eq!(prefix.description(), "foo");
        assert_eq!(prefix.lease_duration(), Some(86400));
        assert_eq!(prefix.dns_flags(), Some(&vec!(String::from("enabled"))));
        assert_eq!(prefix.routers(), Some(vec!("10.112.130.1".parse().unwrap())));
        assert_eq!(prefix.dns_domain(), Some(&String::from("example.com")));
        assert_eq!(prefix.dns_servers(), Some(vec!("10.112.130.2".parse().unwrap(), "10.112.130.3".parse().unwrap())));
    }
}
