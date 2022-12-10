use std::{fmt, net::Ipv4Addr, ptr, ffi::c_void, collections::HashMap};
use serde::Deserialize;
use windows::{
    Win32::NetworkManagement::Dhcp::*,
    core::{HSTRING, PWSTR, PCWSTR},
};

use super::{WinDhcpError, WinDhcpResult};

mod options;
use self::options::*;

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

        match unsafe { DhcpGetSubnetInfo(&self.serveripaddress, self.subnetaddress, &mut subnetinfo) } {
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

    fn get_elements(&self, enumelementtype: DHCP_SUBNET_ELEMENT_TYPE) -> Result<Option<DHCP_SUBNET_ELEMENT_INFO_ARRAY_V5>, u32> {
        let mut resumehandle: u32 = 0;
        let mut elementsread: u32 = 0;
        let mut elementstotal: u32 = 0;

        let mut enumelementinfo: *mut DHCP_SUBNET_ELEMENT_INFO_ARRAY_V5 = ptr::null_mut();

        match unsafe { DhcpEnumSubnetElementsV5(&self.serveripaddress, self.subnetaddress,
            enumelementtype,
            &mut resumehandle,
            0xFFFFFFFF,
            &mut enumelementinfo,
            &mut elementsread,
            &mut elementstotal)} {
                0 => (),
                //ERROR_NO_MORE_ITEMS
                259 => { return Ok(None); },
                n => { return Err(n); },
        }

        let data: DHCP_SUBNET_ELEMENT_INFO_ARRAY_V5 = unsafe {*enumelementinfo};

        unsafe { DhcpRpcFreeMemory(enumelementinfo as *mut c_void); };

        Ok(Some(data))
    }

    fn get_first_element(&self, enumelementtype: DHCP_SUBNET_ELEMENT_TYPE) -> Result<DHCP_SUBNET_ELEMENT_DATA_V5, u32> {
        let mut resumehandle: u32 = 0;
        let mut elementsread: u32 = 0;
        let mut elementstotal: u32 = 0;

        let mut enumelementinfo: *mut DHCP_SUBNET_ELEMENT_INFO_ARRAY_V5 = ptr::null_mut();

        match unsafe { DhcpEnumSubnetElementsV5(&self.serveripaddress, self.subnetaddress,
            enumelementtype,
            &mut resumehandle,
            0xFFFFFFFF,
            &mut enumelementinfo,
            &mut elementsread,
            &mut elementstotal)} {
                0 => (),
                n => { return Err(n); },
        }

        let data: DHCP_SUBNET_ELEMENT_DATA_V5 = unsafe {*(*enumelementinfo).Elements};

        if unsafe {(*enumelementinfo).NumElements} > 1 {
            for idx in 1..unsafe {(*enumelementinfo).NumElements} {
                let ptr = unsafe {(*enumelementinfo).Elements.offset(idx.try_into().unwrap())};
                unsafe { DhcpRpcFreeMemory(ptr as *mut c_void); };
            }
        }

        unsafe { DhcpRpcFreeMemory((*enumelementinfo).Elements as *mut c_void); };
        unsafe { DhcpRpcFreeMemory(enumelementinfo as *mut c_void); };

        Ok(data)
    }

    fn add_element(&self, addelementinfo: &DHCP_SUBNET_ELEMENT_DATA_V5) -> Result<(), u32> {
        match unsafe { DhcpAddSubnetElementV5(&self.serveripaddress, self.subnetaddress, addelementinfo) } {
            0 => Ok(()),
            n => Err(n),
        }
    }
/*
    fn get_option_u32(&self, optionid: u32) -> Result<u32, u32> {
        let mut optionvalue: *mut DHCP_OPTION_VALUE = ptr::null_mut();

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        match unsafe { DhcpGetOptionValueV5(&self.serveripaddress,
            0x00,
            optionid,
            PCWSTR::null(),
            PCWSTR::null(),
            &mut scopeinfo,
            &mut optionvalue) } {
                0 => (),
                n => { return Err(n); },
        }

        let option = unsafe{ (*((*optionvalue).Value.Elements)).Element.DWordOption };

        unsafe { DhcpRpcFreeMemory((*optionvalue).Value.Elements as *mut c_void) };
        unsafe { DhcpRpcFreeMemory(optionvalue as *mut c_void) };

        Ok(option)
    }

    fn set_option_u32(&self, optionid: u32, value: u32) -> Result<(), u32> {
        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        let mut value = DHCP_OPTION_DATA_ELEMENT {
            OptionType: DhcpDWordOption,
            Element: DHCP_OPTION_DATA_ELEMENT_0 {
                DWordOption: value
            },
        };

        let mut optionvalue = DHCP_OPTION_DATA {
            NumElements: 1,
            Elements: &mut value
        };

        match unsafe { DhcpSetOptionValueV5(&self.serveripaddress,
            0x00,
            optionid,
            PCWSTR::null(),
            PCWSTR::null(),
            &mut scopeinfo,
            &mut optionvalue,
        ) } {
            0 => Ok(()),
            n => Err(n),
        }
    }
    
    fn get_option_ip(&self, optionid: u32) -> Result<Ipv4Addr, u32> {
        let mut optionvalue: *mut DHCP_OPTION_VALUE = ptr::null_mut();

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        match unsafe { DhcpGetOptionValueV5(&self.serveripaddress,
            0x00,
            optionid,
            PCWSTR::null(),
            PCWSTR::null(),
            &mut scopeinfo,
            &mut optionvalue) } {
                0 => (),
                n => { return Err(n); },
        }

        let option = unsafe{ (*((*optionvalue).Value.Elements)).Element.IpAddressOption };

        unsafe { DhcpRpcFreeMemory((*optionvalue).Value.Elements as *mut c_void) };
        unsafe { DhcpRpcFreeMemory(optionvalue as *mut c_void) };

        Ok(Ipv4Addr::from(option))
    }
    
    fn get_option_ips(&self, optionid: u32) -> Result<Vec<Ipv4Addr>, u32> {
        let mut optionvalue: *mut DHCP_OPTION_VALUE = ptr::null_mut();

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        match unsafe { DhcpGetOptionValueV5(&self.serveripaddress,
            0x00,
            optionid,
            PCWSTR::null(),
            PCWSTR::null(),
            &mut scopeinfo,
            &mut optionvalue) } {
                0 => (),
                2 => return Ok(Vec::new()),
                e => return Err(e),
        }

        let len = unsafe{ (*optionvalue).Value.NumElements };

        let mut ips = Vec::with_capacity(len as usize);

        for idx in 0..len {
            let element = unsafe{ (*optionvalue).Value.Elements.offset(idx.try_into().unwrap()) };
            let value = unsafe{ (*element).Element.IpAddressOption };
            ips.push(Ipv4Addr::from(value));
            unsafe { DhcpRpcFreeMemory(element as *mut c_void) };
        }

        unsafe { DhcpRpcFreeMemory(optionvalue as *mut c_void) };

        Ok(ips)
    }

    fn set_option_ip(&self, optionid: u32, value: Ipv4Addr) -> Result<(), u32> {
        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        let mut value = DHCP_OPTION_DATA_ELEMENT {
            OptionType: DhcpIpAddressOption,
            Element: DHCP_OPTION_DATA_ELEMENT_0 {
                IpAddressOption: u32::from(value)
            },
        };

        let mut optionvalue = DHCP_OPTION_DATA {
            NumElements: 1,
            Elements: &mut value
        };

        match unsafe { DhcpSetOptionValueV5(&self.serveripaddress,
            0x00,
            optionid,
            PCWSTR::null(),
            PCWSTR::null(),
            &mut scopeinfo,
            &mut optionvalue,
        ) } {
            0 => Ok(()),
            n => Err(n),
        }
    }

    fn set_option_ips(&self, optionid: u32, values: &Vec<Ipv4Addr>) -> Result<(), u32> {
        if values.is_empty() {
            return self.remove_option(optionid);
        }

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        let mut values = values.iter().map( |i|
            DHCP_OPTION_DATA_ELEMENT {
                OptionType: DhcpIpAddressOption,
                Element: DHCP_OPTION_DATA_ELEMENT_0 {
                    IpAddressOption: u32::from(i.to_owned())
                },
            }
        ).collect::<Vec<DHCP_OPTION_DATA_ELEMENT>>();

        let mut optionvalue = DHCP_OPTION_DATA {
            NumElements: values.len() as u32,
            Elements: values.as_mut_ptr(),
        };

        match unsafe { DhcpSetOptionValueV5(&self.serveripaddress,
            0x00,
            optionid,
            PCWSTR::null(),
            PCWSTR::null(),
            &mut scopeinfo,
            &mut optionvalue,
        ) } {
            0 => Ok(()),
            n => Err(n),
        }
    }
*/
    fn remove_option(&self, optionid: u32) -> Result<(), u32> {
        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        match unsafe { DhcpRemoveOptionValueV5(&self.serveripaddress,
            0x00,
            optionid,
            PCWSTR::null(),
            PCWSTR::null(),
            &mut scopeinfo,
        ) } {
            0 => Ok(()),
            n => Err(n),
        }
    }

    pub fn get(serveripaddress: &HSTRING, subnetaddress: &Ipv4Addr) -> Result<Option<Self>, u32> {
        let mut subnetinfo: *mut DHCP_SUBNET_INFO = std::ptr::null_mut();
        let subnetaddress = u32::from(*subnetaddress);

        let ret = match unsafe { DhcpGetSubnetInfo(serveripaddress, subnetaddress, &mut subnetinfo) } {
            0 => {
                let subnet_name = match unsafe{(*subnetinfo).SubnetName}.is_null() {
                    true => String::default(),
                    false => unsafe{(*subnetinfo).SubnetName.to_string()}.unwrap_or_default(),
                };
                let subnet_comment = match unsafe{(*subnetinfo).SubnetComment}.is_null() {
                    true => String::default(),
                    false => unsafe{(*subnetinfo).SubnetComment.to_string()}.unwrap_or_default(),
                };

                Ok(Some(Self {
                    serveripaddress: serveripaddress.clone(),
                    subnetaddress,
                    subnet_mask: Ipv4Addr::from(unsafe{*subnetinfo}.SubnetMask),
                    subnet_name,
                    subnet_comment,
                }))
            },
            ERROR_DHCP_SUBNET_NOT_PRESENT => Ok(None),
            n => Err(n),
        };

        unsafe { DhcpRpcFreeMemory(subnetinfo as *mut c_void) };

        ret
    }

    pub fn create(serveripaddress: &HSTRING, subnetaddress: &Ipv4Addr, subnetmask: &Ipv4Addr) -> Result<Self, u32> {
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

        #[cfg(windows)]
        let mut wname: windy::WString = name.try_into().unwrap();
        #[cfg(windows)]
        let wptr = wname.as_mut_c_str().as_mut_ptr();
        #[cfg(not(windows))]
        let wptr = name.encode_utf16().chain([0u16]).collect::<Vec<u16>>().as_mut_ptr();

        subnetinfo.SubnetName = PWSTR::from_raw(wptr);

        self.set_subnet_info(subnetinfo)
            .map_err(|e| WinDhcpError::new("setting subnet name", e))
    }

    pub fn set_comment(&self, comment: &str) -> WinDhcpResult<()> {
        let mut subnetinfo = self.get_subnet_info()
        .map_err(|e| WinDhcpError::new("setting subnet comment", e))?;

        #[cfg(windows)]
        let mut wcomment: windy::WString = comment.try_into().unwrap();
        #[cfg(windows)]
        let wptr = wcomment.as_mut_c_str().as_mut_ptr();
        #[cfg(not(windows))]
        let wptr = comment.encode_utf16().chain([0u16]).collect::<Vec<u16>>().as_mut_ptr();

        subnetinfo.SubnetComment = PWSTR::from_raw(wptr);
        self.set_subnet_info(subnetinfo)
            .map_err(|e| WinDhcpError::new("setting subnet comment", e))
    }

    pub fn get_subnet_range(&self) -> WinDhcpResult<(Ipv4Addr, Ipv4Addr)> {

        let data = match self.get_first_element(DhcpIpRangesDhcpBootp) {
            Ok(data) => data,
            //ERROR_NO_MORE_ITEMS
            Err(259) => { return Ok((Ipv4Addr::from(0), Ipv4Addr::from(0))); },
            Err(e) => return Err(WinDhcpError::new("getting subnet range", e)),
        };

        let ret = (
            Ipv4Addr::from(unsafe {(*data.Element.IpRange).StartAddress}),
            Ipv4Addr::from(unsafe {(*data.Element.IpRange).EndAddress})
        );
        
        //unsafe { DhcpRpcFreeMemory(&mut data as *mut c_void) };

        Ok(ret)
    }

    pub fn set_subnet_range(&self, start_address: Ipv4Addr, end_address: Ipv4Addr) -> WinDhcpResult<()> {
        let start_address = u32::from(start_address);
        let end_address = u32::from(end_address);

        let mut data = match self.get_first_element(DhcpIpRangesDhcpBootp) {
            Ok(data) => data,
            //ERROR_NO_MORE_ITEMS
            Err(259) => {
                let mut ip_range = DHCP_BOOTP_IP_RANGE {
                    StartAddress: std::u32::MAX,
                    EndAddress: 0u32,
                    BootpAllocated: 0u32,
                    MaxBootpAllowed: 0u32,
                };
                DHCP_SUBNET_ELEMENT_DATA_V5 {
                    ElementType: DhcpIpRangesDhcpBootp,
                    Element: DHCP_SUBNET_ELEMENT_DATA_V5_0 {
                        IpRange: &mut ip_range,
                    },
                }
            },
            Err(e) => return Err(WinDhcpError::new("getting subnet range", e)),
        };
        
        unsafe {
            (*data.Element.IpRange).StartAddress = std::cmp::min((*data.Element.IpRange).StartAddress, start_address);
            (*data.Element.IpRange).EndAddress = std::cmp::max((*data.Element.IpRange).EndAddress, end_address);
        }

        self.add_element(&data)
            .map_err(|e| WinDhcpError::new("setting subnet range2", e))?;
        
        unsafe {
            (*data.Element.IpRange).StartAddress = start_address;
            (*data.Element.IpRange).EndAddress = end_address;
        }

        self.add_element(&data)
            .map_err(|e| WinDhcpError::new("setting subnet range3", e))?;

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
        Ok(self.get_option(81)?
            .map(|f: u32| DnsFlags::from(f)))
    }

    pub fn set_dns_flags(&self, dns_flags: Option<&DnsFlags>) -> WinDhcpResult<()> {
        self.set_option(81, dns_flags.map(u32::from).as_ref())
    }

    pub fn get_routers(&self) -> WinDhcpResult<Vec<Ipv4Addr>> {
        self.get_options(OPTION_ROUTER_ADDRESS)
    }

    pub fn set_routers(&self, routers: &Vec<Ipv4Addr>) -> WinDhcpResult<()> {
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

    pub fn set_dns_servers(&self, servers: &Vec<Ipv4Addr>) -> WinDhcpResult<()> {
        self.set_options(OPTION_DOMAIN_NAME_SERVERS, servers)
    }
    
    pub fn get_reservations(&self) -> Result<HashMap<Ipv4Addr, Vec<u8>>, u32> {
        let reservations = self.get_elements(DhcpReservedIps)?;
        if reservations.is_none() { return Ok(HashMap::new()) }
        let reservations = reservations.unwrap();

        let mut ret = HashMap::with_capacity(reservations.NumElements as usize);

        for idx in 0..reservations.NumElements {

            let reservation = unsafe { *(*reservations.Elements.offset(idx.try_into().unwrap())).Element.ReservedIp };

            let vec_len = unsafe{(*reservation.ReservedForClient).DataLength} as usize;

            ret.insert(Ipv4Addr::from(reservation.ReservedIpAddress), unsafe { Vec::from_raw_parts((*reservation.ReservedForClient).Data, vec_len, vec_len)[5..].to_vec().clone() });
        }
        unsafe { DhcpRpcFreeMemory(reservations.Elements as *mut c_void); };
        
        Ok(ret)
    }

    pub fn add_reservation(&self, reservationaddress: Ipv4Addr, macaddress: &Vec<u8>) -> WinDhcpResult<()> {
        let mut reserved_for_client = DHCP_BINARY_DATA {
            DataLength: macaddress.len() as u32,
            Data: macaddress.clone().as_mut_ptr(),
        };
        let mut reserved_ip = DHCP_IP_RESERVATION_V4 {
            ReservedIpAddress: u32::from(reservationaddress),
            ReservedForClient: &mut reserved_for_client,
            bAllowedClientTypes: 3
        };
        let addelementinfo = DHCP_SUBNET_ELEMENT_DATA_V5{
            ElementType: DhcpReservedIps,
            Element: DHCP_SUBNET_ELEMENT_DATA_V5_0 {
                ReservedIp: &mut reserved_ip
            }
        };

        match unsafe { DhcpAddSubnetElementV5(&self.serveripaddress, self.subnetaddress, &addelementinfo) } {
            0 => Ok(()),
            e => Err(WinDhcpError::new("adding reservation", e)),
        }
    }

    pub fn remove_reservation(&self, reservationaddress: Ipv4Addr, macaddress: &Vec<u8>) -> WinDhcpResult<()> {
        let mut data: Vec<u8> = Ipv4Addr::from(self.subnetaddress).octets().into_iter().rev()
            .chain(::std::iter::once(0x01))
            .chain(macaddress.clone().into_iter())
            .collect();

        let mut reserved_for_client = DHCP_BINARY_DATA {
            DataLength: (macaddress.len() + 5) as u32,
            Data: data.as_mut_ptr(),
        };
        let mut reserved_ip = DHCP_IP_RESERVATION_V4 {
            ReservedIpAddress: u32::from(reservationaddress),
            ReservedForClient: &mut reserved_for_client,
            bAllowedClientTypes: 3
        };
        let removeelementinfo = DHCP_SUBNET_ELEMENT_DATA_V5{
            ElementType: DhcpReservedIps,
            Element: DHCP_SUBNET_ELEMENT_DATA_V5_0 {
                ReservedIp: &mut reserved_ip
            }
        };

        match unsafe { DhcpRemoveSubnetElementV5(&self.serveripaddress, self.subnetaddress, &removeelementinfo, DhcpFullForce) } {
                0 => Ok(()),
                e => Err(WinDhcpError::new("removing reservation", e)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Reservation {
    pub ipaddress: Ipv4Addr,
    pub mac: Vec<u8>,
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
            update_downlevel:  flags & DNS_FLAG_UPDATE_DOWNLEVEL == DNS_FLAG_UPDATE_DOWNLEVEL,
            cleanup_expired:  flags & DNS_FLAG_CLEANUP_EXPIRED == DNS_FLAG_CLEANUP_EXPIRED,
            update_both_always:  flags & DNS_FLAG_UPDATE_BOTH_ALWAYS == DNS_FLAG_UPDATE_BOTH_ALWAYS,
            update_dhcid:  flags & DNS_FLAG_UPDATE_DHCID == DNS_FLAG_UPDATE_DHCID,
            disable_ptr_update:  flags & DNS_FLAG_DISABLE_PTR_UPDATE == DNS_FLAG_DISABLE_PTR_UPDATE,
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