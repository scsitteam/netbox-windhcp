use std::net::Ipv4Addr;

use chrono::NaiveDate;
use ipnet::Ipv4Net;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IpAddress {
    url: String,
    address: Ipv4Net,
    dns_name: String,
    description: String,
    custom_fields: IpAddressCustomField,
    assigned_object: Option<IpAddressAssignedObject>,
}

#[derive(Debug, Deserialize)]
struct IpAddressCustomField {
    dhcp_reservation_mac: Option<String>,
    dhcp_reservation_last_active: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
struct IpAddressAssignedObject {
    url: Option<String>,
}

impl IpAddress {
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }

    pub fn address(&self) -> Ipv4Addr {
        self.address.addr()
    }

    pub fn dns_name(&self) -> &str {
        self.dns_name.as_ref()
    }

    pub fn description(&self) -> &str {
        self.description.as_ref()
    }

    pub fn reservation_mac(&self) -> Option<&String> {
        self.custom_fields.dhcp_reservation_mac.as_ref()
    }

    pub fn dhcp_reservation_last_active(&self) -> Option<NaiveDate> {
        self.custom_fields.dhcp_reservation_last_active
    }

    pub fn assigned_object_url(&self) -> Option<&String> {
        match &self.assigned_object {
            Some(ao) => match ao.url.as_ref() {
                Some(url) => Some(url),
                None => None,
            }
            None => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AssignedObject {
    mac_address: Option<String>,
}

impl AssignedObject {
    pub fn mac_address(&self) -> Option<&String> {
        self.mac_address.as_ref()
    }
}