use std::net::Ipv4Addr;

use log::{info, debug, warn};

use crate::sync::mac::MacAddr;
use crate::sync::windhcp::DnsFlags;

use self::netbox::model::*;
use self::windhcp::{WinDhcp, Subnet};
use self::{config::SyncConfig, netbox::NetboxApi};

pub mod config;
mod netbox;
mod windhcp;
mod mac;

pub struct Sync {
    config: SyncConfig,
    netbox: NetboxApi,
    dhcp: WinDhcp,
    noop: bool
}

impl Sync {
    pub fn new(config: SyncConfig, noop: bool) -> Self {
        let netbox = NetboxApi::new(&config.netbox);
        let dhcp = WinDhcp::new(config.dhcp.server());

        Self { config, netbox, dhcp, noop }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + std::marker::Sync>> {
        info!("Start sync from {} to {}", self.config.netbox.apiurl(), self.config.dhcp.server());

        let netbox_version = self.netbox.version().await?;
        debug!("Netbox Version: {}", netbox_version);
        let dhcp_version = self.dhcp.get_version()?;
        debug!("Windows DHCp Server Version: {}.{}", dhcp_version.0, dhcp_version.1);

        let prefixes: Vec<Prefix> = self.netbox.get_prefixes().await?;
        let ranges: Vec<IpRange> = self.netbox.get_ranges().await?;
        info!("Found {} Prefixes and {} Ranges", prefixes.len(), ranges.len());

        for prefix in prefixes.iter() {
            info!("Sync Prefix {} - {}", prefix.prefix, prefix.description);
            
            let range = match ranges.iter().find(|&r| prefix.prefix.contains(&r.start_address()) && prefix.prefix.contains(&r.end_address())) {
                Some(r) => r,
                None => {
                    warn!("Skip Prefix {} no range found", prefix.prefix);
                    continue;
                }
            };
            let subnet = self.sync_subnetv4(prefix, range)?;

            /* Update Reservations */
            let mut dhcp_reservations = subnet.get_reservations().unwrap();
            let reservations = self.netbox.get_reservations_for_subnet(&prefix.prefix).await?;
            info!("  Subnet {}: Found {} reservations", &prefix.addr(), reservations.len());

            for reservation in reservations.iter() {
                self.sync_reservationv4(&subnet, reservation,  dhcp_reservations.remove(&reservation.address())).await?;
            }

            /* Cleanup old Reservations */
            for (reservationaddress, macaddress) in dhcp_reservations {
                if !self.noop { subnet.remove_reservation(reservationaddress, &macaddress)?; }
                info!("  Reservation {}: Remove Reservation {}", &reservationaddress, &macaddress.as_mac());
            }
        }

        /* Cleanup old Subnets */
        let prefixes_ip: Vec<Ipv4Addr> = prefixes.iter().map(|i| i.addr()).collect();
        for subnet in self.dhcp.get_subnets()? {
            if prefixes_ip.contains(&subnet) {
                continue;
            }
            
            if !self.noop { self.dhcp.remove_subnet(subnet)?; }
            info!("Subnet {}: Removed", &subnet);
        }

        Ok(())
    }

    fn sync_subnetv4(&self, prefix: &Prefix, range: &IpRange) -> Result<Subnet, Box<dyn std::error::Error + Send + std::marker::Sync>> {
        let subnetaddress = &prefix.addr();

        let subnet = self.dhcp.get_or_create_subnet(subnetaddress, &prefix.netmask()).unwrap();
        debug!("Found: {} - {}", subnetaddress, subnet.subnet_name);

        /* Subnet Netmask */
        if subnet.subnet_mask != prefix.netmask() {
            if !self.noop { subnet.set_mask(prefix.netmask())?; }
            info!("  Subnet {}: Updated netmask to {}", &subnetaddress, prefix.netmask());
        }
        
        /* Subnet Name */
        if subnet.subnet_name != prefix.description {
            if !self.noop { subnet.set_name(&prefix.description)?; }
            info!("  Subnet {}: Updated name to {}", &subnetaddress, &prefix.description);
        }
        
        /* Subnet Comment */
        if subnet.subnet_comment != prefix.description {
            if !self.noop { subnet.set_comment(&prefix.description)?; }
            info!("  Subnet {}: Updated comment to {}", &subnetaddress, &prefix.description);
        }

        /* DHCP Range */
        if (range.start_address(), range.end_address()) != subnet.get_subnet_range()? {
            if !self.noop { subnet.set_subnet_range(range.start_address(), range.end_address())?; }
            info!("  Subnet {}: Updated range to {}-{}", &subnetaddress, range.start_address(), range.end_address());
        }

        /* Lease Duration */
        let lease_duration = prefix.custom_fields.dhcp_lease_duration
            .unwrap_or_else(|| self.config.dhcp.lease_duration());
        if lease_duration != subnet.get_lease_duration()? {
            if !self.noop { subnet.set_lease_duration(lease_duration)?; }
            info!("  Subnet {}: Updated lease duration to {}", &subnetaddress, lease_duration);
        }
        
        /* DNS Update */
        let dns_flags = prefix.custom_fields.dhcp_dns_flags()
            .map_or_else(|| self.config.dhcp.default_dns_flags().to_owned(), DnsFlags::from);
        if dns_flags != subnet.get_dns_flags()? {
            if !self.noop { subnet.set_dns_flags(&dns_flags)?; }
            info!("  Subnet {}: Updated dns flags to {:?}", &subnetaddress, dns_flags);
        }

        Ok(subnet)
    }

    async fn sync_reservationv4(&self, subnet: &Subnet, reservation: &IpAddress, dhcp_mac: Option<Vec<u8>>) -> Result<(), Box<dyn std::error::Error + Send + std::marker::Sync>> {
        let mac = match self.get_macaddress_for_reservation(reservation).await? {
            Some(mac) => mac,
            None => {
                warn!("Error no MAC address found for IP {}", &reservation.address());
                return Ok(());
            },
        };

        /* Reservation */
        if let Some(macaddress) = dhcp_mac {
            if macaddress != mac {
                if !self.noop {
                    subnet.remove_reservation(reservation.address(), &macaddress)?;
                    subnet.add_reservation(reservation.address(), &mac)?;
                }
                info!("  Reservation {}: Update Reservation {:?}", &reservation.address(), &mac.as_mac());
            }
        } else {
            if !self.noop { subnet.add_reservation(reservation.address(), &mac)?; }
            info!("  Reservation {}: Create Reservation {:?}", &reservation.address(), &mac.as_mac());
        }

        /* Client Name */
        let name = self.dhcp.get_client_name(reservation.address()).unwrap_or_default();
        if name != reservation.dns_name() {
            if !self.noop { self.dhcp.set_client_name(reservation.address(), reservation.dns_name())?; }
            info!("  Reservation {}: Set client name to {}", &reservation.address(), &reservation.dns_name());
        }

        /* Client Comment */
        let comment = self.dhcp.get_client_comment(reservation.address()).unwrap_or_default();
        if comment != reservation.description() {
            if !self.noop { self.dhcp.set_client_comment(reservation.address(), reservation.description())?; }
            info!("  Reservation {}: Set client comment to {}", &reservation.address(), &reservation.description());
        }

        Ok(())
    }

    async fn get_macaddress_for_reservation(&self, reservation: &IpAddress) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + std::marker::Sync>> {
        let mac = match reservation.custom_fields().dhcp_reservation_mac() {
            Some(mac) => mac.clone(),
            None => match self.get_macaddress_for_reservation_from_assigned_object(reservation.assigned_object()).await? {
                Some(mac) => mac,
                None => return Ok(None),
            }
        };

        Ok(Some(Vec::<u8>::from_mac(&mac)))
    }

    async fn get_macaddress_for_reservation_from_assigned_object(&self, object: Option<&IpAddressAssignedObject>) -> Result<Option<String>, Box<dyn std::error::Error + Send + std::marker::Sync>> {
        let url = match object.and_then(|o| o.url().cloned()) {
            Some(url) => url,
            None => return Ok(None),
        };

        Ok(self.netbox.get_object::<AssignedObject>(url.as_str()).await?.mac_address().cloned())
    }
}