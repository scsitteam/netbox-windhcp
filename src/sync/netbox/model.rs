use std::net::Ipv4Addr;

use ipnet::Ipv4Net;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Pageination<T> {
    pub count: usize,
    pub next: Option<String>,
    pub results: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct Prefix {
    pub description: String,
    pub prefix: Ipv4Net,
    pub custom_fields: PrefixCustomField
}

impl Prefix {
    pub fn addr(&self) -> Ipv4Addr {
        self.prefix.addr()
    }
    pub fn netmask(&self) -> Ipv4Addr {
        self.prefix.netmask()
    }
}

#[derive(Debug, Deserialize)]
pub struct PrefixCustomField {
    pub dhcp_lease_duration: Option<u32>,
    dhcp_dns_flags: Option<Vec<String>>,
    dhcp_router: Option<Ipv4Addr>,
    dhcp_dns_domain: Option<String>,
    dhcp_dns_servers: Option<Vec<PrefixCustomFieldIp>>,
}


#[derive(Debug, Deserialize)]
pub struct PrefixCustomFieldIp {
    pub address: Ipv4Net
}

impl PrefixCustomField {
    pub fn dhcp_dns_flags(&self) -> Option<&Vec<String>> {
        self.dhcp_dns_flags.as_ref()
    }

    pub fn dhcp_router(&self) -> Option<Ipv4Addr> {
        self.dhcp_router
    }

    pub fn dhcp_dns_domain(&self) -> Option<&String> {
        self.dhcp_dns_domain.as_ref()
    }

    pub fn dhcp_dns_servers(&self) -> Option<Vec<Ipv4Addr>> {
        match &self.dhcp_dns_servers {
            Some(dns) => Some(dns.iter().map(|n| n.address.addr()).collect::<Vec<Ipv4Addr>>()),
            None => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct IpRange {
    pub display: String,
    pub description: String,
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
}

#[derive(Debug, Deserialize)]
pub struct IpAddress {
    address: Ipv4Net,
    dns_name: String,
    description: String,
    custom_fields: IpAddressCustomField,
    assigned_object: Option<IpAddressAssignedObject>,
}

#[derive(Debug, Deserialize)]
pub struct IpAddressCustomField {
    dhcp_reservation_mac: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IpAddressAssignedObject {
    url: Option<String>,
}

impl IpAddressAssignedObject {
    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }
}

impl IpAddressCustomField {
    pub fn dhcp_reservation_mac(&self) -> Option<&String> {
        self.dhcp_reservation_mac.as_ref()
    }
}

impl IpAddress {
    pub fn address(&self) -> Ipv4Addr {
        self.address.addr()
    }

    pub fn custom_fields(&self) -> &IpAddressCustomField {
        &self.custom_fields
    }

    pub fn assigned_object(&self) -> Option<&IpAddressAssignedObject> {
        self.assigned_object.as_ref()
    }

    pub fn dns_name(&self) -> &str {
        self.dns_name.as_ref()
    }

    pub fn description(&self) -> &str {
        self.description.as_ref()
    }
}

#[derive(Debug, Deserialize)]
pub struct AssignedObject {
    mac_address: Option<String>
}

impl AssignedObject {
    pub fn mac_address(&self) -> Option<&String> {
        self.mac_address.as_ref()
    }
}