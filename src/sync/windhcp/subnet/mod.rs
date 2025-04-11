use ipnet::Ipv4Net;
use log::{info, trace};
use serde::Deserialize;
use std::{collections::HashMap, fmt, net::Ipv4Addr, ptr};
use windows::{
    core::{HSTRING, PCWSTR, PWSTR},
    Win32::NetworkManagement::Dhcp::*,
};

use super::{WinDhcpError, WinDhcpResult};

mod options;
use self::options::*;
mod elements;
use self::elements::*;
pub mod reservation;
use self::reservation::*;

#[derive(Debug)]
pub struct Subnet {
    serveripaddress: HSTRING,
    pub subnetaddress: u32,
    pub subnet_mask: Ipv4Addr,
    pub subnet_name: String,
    pub subnet_comment: String,
}

impl Subnet {

    fn get_subnet_info(&self) -> Result<DHCP_SUBNET_INFO, u32> {
        let mut subnetinfo: *mut DHCP_SUBNET_INFO = ptr::null_mut();

        match unsafe {
            trace!("Call DhcpGetSubnetInfo({}, {}, ptr)", &self.serveripaddress, self.subnetaddress);
            DhcpGetSubnetInfo(&self.serveripaddress, self.subnetaddress, &mut subnetinfo)
        } {
            0 => Ok(unsafe{*subnetinfo}),
            n => Err(n),
        }
    }

    fn set_subnet_info(&self, subnetinfo: DHCP_SUBNET_INFO) -> Result<(), u32> {
        match unsafe { DhcpSetSubnetInfo(&self.serveripaddress, self.subnetaddress, &subnetinfo) } {
            0 => Ok(()),
            n => Err(n),
        }
    }

    fn remove_option(&self, optionid: u32) -> Result<(), u32> {
        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        match unsafe {
            DhcpRemoveOptionValueV5(
                &self.serveripaddress,
                0x00,
                optionid,
                PCWSTR::null(),
                PCWSTR::null(),
                &mut scopeinfo,
            )
        } {
            0 => Ok(()),
            n => Err(n),
        }
    }

    pub fn get(serveripaddress: &HSTRING, subnetaddress: &Ipv4Addr) -> Result<Option<Self>, u32> {
        let mut subnetinfo: *mut DHCP_SUBNET_INFO = std::ptr::null_mut();
        let subnetaddress = u32::from(*subnetaddress);

        let ret = match unsafe { DhcpGetSubnetInfo(serveripaddress, subnetaddress, &mut subnetinfo) } {
            0 => {
                let subnet_name = match unsafe { (*subnetinfo).SubnetName }.is_null() {
                    true => String::default(),
                    false => unsafe { (*subnetinfo).SubnetName.to_string() }.unwrap_or_default(),
                };
                let subnet_comment = match unsafe { (*subnetinfo).SubnetComment }.is_null() {
                    true => String::default(),
                    false => unsafe { (*subnetinfo).SubnetComment.to_string() }.unwrap_or_default(),
                };

                Ok(Some(Self {
                    serveripaddress: serveripaddress.clone(),
                    subnetaddress,
                    subnet_mask: Ipv4Addr::from(unsafe { *subnetinfo }.SubnetMask),
                    subnet_name,
                    subnet_comment,
                }))
            },
            ERROR_DHCP_SUBNET_NOT_PRESENT => Ok(None),
            n => Err(n),
        };

        #[cfg(feature = "rpc_free")]
        unsafe { DhcpRpcFreeMemory(subnetinfo as *mut c_void) };

        ret
    }

    pub fn create(
        serveripaddress: &HSTRING,
        subnetaddress: &Ipv4Addr,
        subnetmask: &Ipv4Addr,
    ) -> Result<Self, u32> {
        let subnetinfo = DHCP_SUBNET_INFO {
            SubnetAddress: u32::from(*subnetaddress),
            SubnetMask: u32::from(*subnetmask),
            SubnetName: PWSTR::null(),
            SubnetComment: PWSTR::null(),
            PrimaryHost: DHCP_HOST_INFO::default(),
            SubnetState: DhcpSubnetEnabled,
        };

        match unsafe { DhcpCreateSubnet(serveripaddress, u32::from(*subnetaddress), &subnetinfo) } {
            0 => match Self::get(serveripaddress, subnetaddress)? {
                Some(subnet) => Ok(subnet),
                None => Err(0)
            },
            n => Err(n),
        }
    }

    pub fn set_mask(&self, subnetmask: Ipv4Addr) -> WinDhcpResult<()> {
        let mut subnetinfo = self.get_subnet_info()
            .map_err(|e| WinDhcpError::new("setting subnet mask", e))?;

        subnetinfo.SubnetMask = u32::from(subnetmask);
        self.set_subnet_info(subnetinfo)
            .map_err(|e| WinDhcpError::new("setting subnet mask", e))?;
        Ok(())
    }

    pub fn set_name(&self, name: &str) -> WinDhcpResult<()> {
        let mut subnetinfo = self.get_subnet_info()
            .map_err(|e| WinDhcpError::new("setting subnet name", e))?;

        let mut wname = name.encode_utf16().chain([0u16]).collect::<Vec<u16>>();
        subnetinfo.SubnetName = PWSTR(wname.as_mut_ptr());

        self.set_subnet_info(subnetinfo)
            .map_err(|e| WinDhcpError::new("setting subnet name", e))
    }

    pub fn set_comment(&self, comment: &str) -> WinDhcpResult<()> {
        let mut subnetinfo = self.get_subnet_info()
        .map_err(|e| WinDhcpError::new("setting subnet comment", e))?;

        let mut wcomment = comment.encode_utf16().chain([0u16]).collect::<Vec<u16>>();
        subnetinfo.SubnetComment = PWSTR(wcomment.as_mut_ptr());
        self.set_subnet_info(subnetinfo)
            .map_err(|e| WinDhcpError::new("setting subnet comment", e))
    }

    pub fn get_subnet_range(&self) -> WinDhcpResult<(Ipv4Addr, Ipv4Addr)> {
        match SubnetElements::<DHCP_BOOTP_IP_RANGE>::get_first_element(self) {
            Ok(Some(range)) => Ok((Ipv4Addr::from(range.StartAddress), Ipv4Addr::from(range.EndAddress))),
            Ok(None) => Ok((Ipv4Addr::from(0), Ipv4Addr::from(0))),
            Err(e) => Err(WinDhcpError::new("getting subnet range", e)),
        }
    }

    pub fn set_subnet_range(
        &self,
        start_address: Ipv4Addr,
        end_address: Ipv4Addr,
    ) -> WinDhcpResult<()> {
        let start_address = u32::from(start_address);
        let end_address = u32::from(end_address);

        let mut range = match SubnetElements::<DHCP_BOOTP_IP_RANGE>::get_first_element(self) {
            Ok(Some(range)) => range,
            Ok(None) => DHCP_BOOTP_IP_RANGE {
                StartAddress: std::u32::MAX,
                EndAddress: 0u32,
                BootpAllocated: 0u32,
                MaxBootpAllowed: 0u32,
            },
            Err(e) => return Err(WinDhcpError::new("getting subnet range", e)),
        };

        range.StartAddress = std::cmp::max(
            std::cmp::min(range.StartAddress, start_address),
            self.get_range_min()
        );

        range.EndAddress = std::cmp::min(
            std::cmp::max(range.EndAddress, end_address),
            self.get_range_max()
        );
        info!("Set range to {} - {}", Ipv4Addr::from(range.StartAddress), Ipv4Addr::from(range.EndAddress));

        self.add_element(&mut range)
            .map_err(|e| WinDhcpError::new("setting subnet range to ", e))?;

        range.StartAddress = start_address;
        range.EndAddress = end_address;
        info!("Set range to {} - {}", Ipv4Addr::from(range.StartAddress), Ipv4Addr::from(range.EndAddress));

        self.add_element(&mut range)
            .map_err(|e| WinDhcpError::new("setting subnet range2", e))?;

        //unsafe { DhcpRpcFreeMemory(data as *mut c_void) };

        Ok(())
    }

    pub fn get_lease_duration(&self) -> WinDhcpResult<Option<u32>> {
        self.get_option(OPTION_LEASE_TIME)
    }

    pub fn set_lease_duration(&self, lease_duration: Option<u32>) -> WinDhcpResult<()> {
        self.set_option(OPTION_LEASE_TIME, lease_duration.as_ref())
    }

    pub fn get_dns_flags(&self) -> WinDhcpResult<Option<DnsFlags>> {
        #[allow(clippy::redundant_closure)]
        Ok(self.get_option(81)?.map(|f: u32| DnsFlags::from(f)))
    }

    pub fn set_dns_flags(&self, dns_flags: Option<&DnsFlags>) -> WinDhcpResult<()> {
        self.set_option(81, dns_flags.map(u32::from).as_ref())
    }

    pub fn get_routers(&self) -> WinDhcpResult<Vec<Ipv4Addr>> {
        self.get_options(OPTION_ROUTER_ADDRESS)
    }

    pub fn set_routers(&self, routers: &[Ipv4Addr]) -> WinDhcpResult<()> {
        self.set_options(OPTION_ROUTER_ADDRESS, routers)
    }

    pub fn get_dns_domain(&self) -> WinDhcpResult<Option<String>> {
        self.get_option(OPTION_DOMAIN_NAME)
    }

    pub fn set_dns_domain(&self, domain: Option<&String>) -> WinDhcpResult<()> {
        self.set_option(OPTION_DOMAIN_NAME, domain)
    }

    pub fn get_dns_servers(&self) -> WinDhcpResult<Vec<Ipv4Addr>> {
        self.get_options(OPTION_DOMAIN_NAME_SERVERS)
    }

    pub fn set_dns_servers(&self, servers: &[Ipv4Addr]) -> WinDhcpResult<()> {
        self.set_options(OPTION_DOMAIN_NAME_SERVERS, servers)
    }

    pub fn get_reservations(&self) -> Result<HashMap<Ipv4Addr, Reservation>, u32> {
        let reservations: Vec<Reservation> = self.get_elements()?;

        if reservations.is_empty() { return Ok(HashMap::new()); }

        let mut ret = HashMap::with_capacity(reservations.len());

        for reservation in reservations.into_iter() {
            ret.insert(reservation.ip_address, reservation);
        }

        Ok(ret)
    }

    pub fn add_reservation(
        &self,
        reservationaddress: Ipv4Addr,
        macaddress: &[u8],
    ) -> WinDhcpResult<()> {
        let mut reservation = Reservation {
            ip_address: reservationaddress,
            for_client: macaddress.to_owned(),
            allowed_client_types: ReservationClientTypes::Both,
        };
        match self.add_element(&mut reservation) {
            Ok(_) => Ok(()),
            Err(e) => Err(WinDhcpError::new("adding reservation", e)),
        }
    }

    pub fn remove_reservation(&self, reservationaddress: Ipv4Addr, macaddress: &[u8]) -> WinDhcpResult<()> {
        let mut reservation = Reservation {
            ip_address: reservationaddress,
            for_client: macaddress.to_owned(),
            allowed_client_types: ReservationClientTypes::Both,
        };
        
        match self.remove_element(&mut reservation) {
            Ok(_) => Ok(()),
            Err(e) => Err(WinDhcpError::new("adding reservation", e)),
        }
    }

    fn get_range_min(&self) -> u32 {
        self.subnetaddress + 1
    } 

    fn get_range_max(&self) -> u32 {
        let net = Ipv4Net::with_netmask(Ipv4Addr::from(self.subnetaddress), self.subnet_mask).expect("Unable to create net");
        net.broadcast().into()
    }

    pub fn get_failover_relationship(&self) -> Result<Option<String>, u32> {
        let mut prelationship: *mut DHCP_FAILOVER_RELATIONSHIP = ptr::null_mut();

        match unsafe {
            trace!("Call DhcpV4FailoverGetScopeRelationship({}, {}, ptr)", &self.serveripaddress, self.subnetaddress);
            DhcpV4FailoverGetScopeRelationship(&self.serveripaddress, self.subnetaddress, &mut prelationship)
        } {
            0 => {
                let name : String = unsafe{(*prelationship).RelationshipName.to_string().unwrap()};
                Ok(Some(name))
            },
            ERROR_DHCP_FO_SCOPE_NOT_IN_RELATIONSHIP => Ok(None),
            n => Err(n),
        }
    }

    pub fn add_failover_relationship(&self, name: &str) -> Result<(), u32> {
        let mut name: Vec<u16> = name.encode_utf16().chain([0u16]).collect::<Vec<u16>>();

        let prelationship = DHCP_FAILOVER_RELATIONSHIP {
            PrimaryServer: 0,
            SecondaryServer: 0,
            Mode: windows::Win32::NetworkManagement::Dhcp::DHCP_FAILOVER_MODE(0),
            ServerType: windows::Win32::NetworkManagement::Dhcp::DHCP_FAILOVER_SERVER(0),
            State: windows::Win32::NetworkManagement::Dhcp::FSM_STATE(0),
            PrevState: windows::Win32::NetworkManagement::Dhcp::FSM_STATE(0),
            Mclt: 0,
            SafePeriod: 0,
            RelationshipName: PWSTR::from_raw(name.as_ptr() as *mut _),
            PrimaryServerName: PWSTR::null(),
            SecondaryServerName: PWSTR::null(),
            pScopes: &mut DHCP_IP_ARRAY {
                NumElements: 1,
                Elements: &mut self.subnetaddress.clone(),
            },
            Percentage: 0,
            SharedSecret: PWSTR::null(),
        };

        match unsafe {
            trace!("Call DhcpV4FailoverAddScopeToRelationship({}, {:?})", &self.serveripaddress, prelationship);
            DhcpV4FailoverAddScopeToRelationship(
                &self.serveripaddress,
                &prelationship
            )
        } {
            0 => {
                Ok(())
            },
            n => Err(n),
        }
    }

    pub fn remove_failover_relationship(&self, name: &str) -> Result<(), u32> {
        let mut name: Vec<u16> = name.encode_utf16().chain([0u16]).collect::<Vec<u16>>();

        let prelationship = DHCP_FAILOVER_RELATIONSHIP {
            PrimaryServer: 0,
            SecondaryServer: 0,
            Mode: windows::Win32::NetworkManagement::Dhcp::DHCP_FAILOVER_MODE(0),
            ServerType: windows::Win32::NetworkManagement::Dhcp::DHCP_FAILOVER_SERVER(0),
            State: windows::Win32::NetworkManagement::Dhcp::FSM_STATE(0),
            PrevState: windows::Win32::NetworkManagement::Dhcp::FSM_STATE(0),
            Mclt: 0,
            SafePeriod: 0,
            RelationshipName: PWSTR::from_raw(name.as_ptr() as *mut _),
            PrimaryServerName: PWSTR::null(),
            SecondaryServerName: PWSTR::null(),
            pScopes: &mut DHCP_IP_ARRAY {
                NumElements: 1,
                Elements: &mut self.subnetaddress.clone(),
            },
            Percentage: 0,
            SharedSecret: PWSTR::null(),
        };

        match unsafe {
            trace!("Call DhcpV4FailoverDeleteScopeFromRelationship({}, {:?})", &self.serveripaddress, prelationship);
            DhcpV4FailoverDeleteScopeFromRelationship(
                &self.serveripaddress,
                &prelationship
            )
        } {
            0 => {
                Ok(())
            },
            n => Err(n),
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
pub struct DnsFlags {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub update_downlevel: bool,
    #[serde(default)]
    pub cleanup_expired: bool,
    #[serde(default)]
    pub update_both_always: bool,
    #[serde(default)]
    pub update_dhcid: bool,
    #[serde(default)]
    pub disable_ptr_update: bool,
}

impl fmt::Display for DnsFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.enabled { write!(f, "Enabled")? } else { write!(f, "Disabled")? }
        if self.update_downlevel { write!(f, ", Update Downlevel")? }
        if self.cleanup_expired { write!(f, ", Cleanup Expired")? }
        if self.update_both_always { write!(f, ", Update Both always")? }
        if self.update_dhcid { write!(f, ", Update DHCID")? }
        if self.disable_ptr_update { write!(f, ", Disable PTR update")? }
        Ok(())
    }
}

impl From<&Vec<String>> for DnsFlags {
    fn from(flags: &Vec<String>) -> Self {
        Self {
            enabled: flags.contains(&String::from("enabled")),
            update_downlevel: flags.contains(&String::from("update_downlevel")),
            cleanup_expired: flags.contains(&String::from("cleanup_expired")),
            update_both_always: flags.contains(&String::from("update_both_always")),
            update_dhcid: flags.contains(&String::from("update_dhcid")),
            disable_ptr_update: flags.contains(&String::from("disable_ptr_update")),
        }
    }
}

impl From<u32> for DnsFlags {
    fn from(flags: u32) -> Self {
        Self {
            enabled: flags & DNS_FLAG_ENABLED == DNS_FLAG_ENABLED,
            update_downlevel: flags & DNS_FLAG_UPDATE_DOWNLEVEL == DNS_FLAG_UPDATE_DOWNLEVEL,
            cleanup_expired: flags & DNS_FLAG_CLEANUP_EXPIRED == DNS_FLAG_CLEANUP_EXPIRED,
            update_both_always: flags & DNS_FLAG_UPDATE_BOTH_ALWAYS == DNS_FLAG_UPDATE_BOTH_ALWAYS,
            update_dhcid: flags & DNS_FLAG_UPDATE_DHCID == DNS_FLAG_UPDATE_DHCID,
            disable_ptr_update: flags & DNS_FLAG_DISABLE_PTR_UPDATE == DNS_FLAG_DISABLE_PTR_UPDATE,
        }
    }
}

impl From<&DnsFlags> for u32 {
    fn from(flags: &DnsFlags) -> Self {
        let mut f = 0;

        if flags.enabled { f += DNS_FLAG_ENABLED; }
        if flags.update_downlevel { f += DNS_FLAG_UPDATE_DOWNLEVEL; }
        if flags.cleanup_expired { f += DNS_FLAG_CLEANUP_EXPIRED; }
        if flags.update_both_always { f += DNS_FLAG_UPDATE_BOTH_ALWAYS; }
        if flags.update_dhcid { f += DNS_FLAG_UPDATE_DHCID; }
        if flags.disable_ptr_update { f += DNS_FLAG_DISABLE_PTR_UPDATE; }

        f
    }
}