use std::net::Ipv4Addr;

use log::{debug, info, warn};

pub mod config;
use self::netbox::address::{AssignedObject, IpAddress};
use self::netbox::prefix::Prefix;
use self::netbox::range::IpRange;
use self::windhcp::reservation::Reservation;
use self::{config::SyncConfig, netbox::NetboxApi};
mod mac;
use self::mac::MacAddr;
mod netbox;

mod windhcp;
use self::windhcp::{DnsFlags, Subnet, WinDhcp};

pub struct Sync {
    config: SyncConfig,
    netbox: NetboxApi,
    dhcp: WinDhcp,
    noop: bool,
    scope: Option<Ipv4Addr>,
}

impl Sync {
    pub fn new(config: SyncConfig, noop: bool, scope: Option<Ipv4Addr>) -> Self {
        let netbox = NetboxApi::new(&config.netbox);
        let dhcp = WinDhcp::new(config.dhcp.server());

        Self { config, netbox, dhcp, noop, scope}
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + std::marker::Sync>> {
        info!("Start sync from {} to {} ({} {})", self.config.netbox.apiurl(), self.config.dhcp.server(), env!("CARGO_PKG_NAME"), git_version::git_version!(prefix = "git:", cargo_prefix = "cargo:", fallback = "unknown"));

        let netbox_version = self.netbox.version()?;
        debug!("Netbox Version: {}", netbox_version);
        let dhcp_version = self.dhcp.get_version()?;
        debug!("Windows DHCp Server Version: {}.{}", dhcp_version.0, dhcp_version.1);

        let prefixes = self.netbox.get_prefixes()?;
        let ranges = self.netbox.get_ranges()?;
        info!("Found {} Prefixes and {} Ranges", prefixes.len(), ranges.len());

        for prefix in prefixes.iter() {
            if let Some(scope) = self.scope {
                if !prefix.prefix().contains(&scope) {
                    debug!("Skip Prefix {} - {}", prefix.prefix(), prefix.description());
                    continue;
                }
            }

            info!("Sync Prefix {} - {}", prefix.prefix(), prefix.description());

            let range = match ranges.iter().find(|&r| r.is_contained(prefix)) {
                Some(r) => r,
                None => {
                    warn!("Skip Prefix {} no range found", prefix.prefix());
                    continue;
                }
            };
            let subnet = self.sync_subnetv4(prefix, range)?;

            /* Update Reservations */
            let mut dhcp_reservations = subnet.get_reservations().unwrap();
            let reservations = self.netbox.get_reservations_for_subnet(&prefix.prefix())?;
            info!("  Subnet {}: Found {} reservations", &prefix.addr(), reservations.len());

            for reservation in reservations.iter() {
                self.sync_reservationv4(&subnet, reservation,  dhcp_reservations.remove(&reservation.address()))?;
            }

            /* Cleanup old Reservations */
            for (reservationaddress, macaddress) in dhcp_reservations {
                if !self.noop { subnet.remove_reservation(reservationaddress, &macaddress.for_client)?; }
                info!("  Reservation {}: Remove Reservation {}", &reservationaddress, &macaddress.for_client.as_mac());
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

    fn sync_subnetv4(
        &self,
        prefix: &Prefix,
        range: &IpRange,
    ) -> Result<Subnet, Box<dyn std::error::Error + Send + std::marker::Sync>> {
        let subnetaddress = &prefix.addr();

        let subnet = self.dhcp.get_or_create_subnet(subnetaddress, &prefix.netmask()).unwrap();
        debug!("Found: {} - {}", subnetaddress, subnet.subnet_name);

        /* Subnet Netmask */
        if subnet.subnet_mask != prefix.netmask() {
            if !self.noop { subnet.set_mask(prefix.netmask())?; }
            info!("  Subnet {}: Updated netmask to {}", &subnetaddress, prefix.netmask());
        }

        /* Subnet Name */
        if subnet.subnet_name != prefix.description() {
            if !self.noop { subnet.set_name(prefix.description())?; }
            info!("  Subnet {}: Updated name to {}", &subnetaddress, prefix.description());
        }

        /* Subnet Comment */
        if subnet.subnet_comment != prefix.description() {
            if !self.noop { subnet.set_comment(prefix.description())?; }
            info!("  Subnet {}: Updated comment to {}", &subnetaddress, prefix.description());
        }

        /* DHCP Range */
        if (range.start_address(), range.end_address()) != subnet.get_subnet_range()? {
            if !self.noop { subnet.set_subnet_range(range.start_address(), range.end_address())?; }
            info!("  Subnet {}: Updated range to {}-{}", &subnetaddress, range.start_address(), range.end_address());
        }

        /* Lease Duration */
        let lease_duration = prefix.lease_duration()
            .or_else(|| Some(self.config.dhcp.lease_duration()));
        if lease_duration != subnet.get_lease_duration()? {
            if !self.noop { subnet.set_lease_duration(lease_duration)?; }
            info!("  Subnet {}: Updated lease duration to {}", &subnetaddress, lease_duration.unwrap_or_default());
        }

        /* DNS Update */
        let dns_flags = prefix.dns_flags()
            .map(DnsFlags::from).or_else(|| self.config.dhcp.default_dns_flags());
        if dns_flags != subnet.get_dns_flags()? {
            if !self.noop { subnet.set_dns_flags(dns_flags.as_ref())?; }
            info!("  Subnet {}: Updated dns flags to {:?}", &subnetaddress, dns_flags);
        }

        /* Router */
        let routers = match prefix.routers() {
            Some(ip) => ip,
            None => {
                let routers = self.netbox.get_router_for_subnet(&prefix.prefix())?;
                routers.iter().map(|i| i.address()).collect()
            }
        };
        if routers != subnet.get_routers()? {
            if !self.noop { subnet.set_routers(&routers)?; }
            info!("  Subnet {}: Updated routers to {:?}", &subnetaddress, routers);
        }

        /* DNS Domain */
        let dns_domain = prefix.dns_domain()
            .or_else(|| self.config.dhcp.default_dns_domain());
        if dns_domain != subnet.get_dns_domain()?.as_ref() {
            if !self.noop { subnet.set_dns_domain(dns_domain)?; }
            info!("  Subnet {}: Updated dns domain to {:?}", &subnetaddress, dns_domain);
        }

        /* DNS Server */
        let dns = prefix.dns_servers()
            .unwrap_or_else(|| self.config.dhcp.default_dns_servers().to_vec());
        if dns != subnet.get_dns_servers()? {
            if !self.noop { subnet.set_dns_servers(&dns)?; }
            info!("  Subnet {}: Updated dns to {:?}", &subnetaddress, dns);
        }

        Ok(subnet)
    }

    fn sync_reservationv4(
        &self,
        subnet: &Subnet,
        reservation: &IpAddress,
        dhcp_reservation: Option<Reservation>,
    ) -> Result<(), Box<dyn std::error::Error + Send + std::marker::Sync>> {
        let mac = match self.get_macaddress_for_reservation(reservation)? {
            Some(mac) => mac,
            None => {
                warn!("Error no MAC address found for IP {}", &reservation.address());
                return Ok(());
            },
        };

        /* Reservation */
        if let Some(macaddress) = dhcp_reservation.map(|r| r.for_client) {
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

    fn get_macaddress_for_reservation(
        &self,
        reservation: &IpAddress,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + std::marker::Sync>> {
        let mac = match reservation.reservation_mac() {
            Some(mac) => Some(mac.clone()),
            None => match reservation.assigned_object_url() {
                Some(url) => self.get_macaddress_for_reservation_from_assigned_object(url)?,
                None => None,
            },
        };

        Ok(mac.map(|m| Vec::<u8>::from_mac(&m)))
    }

    fn get_macaddress_for_reservation_from_assigned_object(&self, url: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + std::marker::Sync>> {
        Ok(self.netbox.get_object::<AssignedObject>(url)?.mac_address().cloned())
    }
}
