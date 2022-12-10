use std::{net::Ipv4Addr, os::raw::c_void, ptr};

use windows::{
    core::{PCWSTR, PWSTR},
    Win32::NetworkManagement::Dhcp::*,
};

use crate::sync::windhcp::{WinDhcpError, WinDhcpResult};

use super::Subnet;

pub trait SubnetOptions<T> {
    fn get_options(&self, optionid: u32) -> WinDhcpResult<Vec<T>>;
    fn set_options(&self, optionid: u32, values: &[T]) -> WinDhcpResult<()>;
}
pub trait SubnetOption<T: Clone>: SubnetOptions<T> {
    fn get_option(&self, optionid: u32) -> WinDhcpResult<Option<T>> {
        let mut values: Vec<T> = self.get_options(optionid)?;
        match values.len() {
            0 => Ok(None),
            1 => Ok(Some(values.remove(0))),
            _ => Err(WinDhcpError::new("multiple values for option only expecting one", 0)),
        }
    }

    fn set_option(&self, optionid: u32, value: Option<&T>) -> WinDhcpResult<()> {
        match value {
            Some(value) => {
                self.set_options(optionid, &[value.clone()])
            },
            None => self.set_options(optionid, &[]),
        }
    }
}

impl SubnetOption<u32> for Subnet {}
impl SubnetOptions<u32> for Subnet {
    fn get_options(&self, optionid: u32) -> WinDhcpResult<Vec<u32>> {
        let mut optionvalue: *mut DHCP_OPTION_VALUE = ptr::null_mut();

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        match unsafe {
            DhcpGetOptionValueV5(
                &self.serveripaddress,
                0x00,
                optionid,
                PCWSTR::null(),
                PCWSTR::null(),
                &mut scopeinfo,
                &mut optionvalue,
            )
        } {
            0 => (),
            2 => return Ok(Vec::new()),
            e => return Err(WinDhcpError::new("getting option", e)),
        }

        let len = unsafe { (*optionvalue).Value.NumElements };

        let mut values = Vec::with_capacity(len as usize);

        for idx in 0..len {
            let element = unsafe{ (*optionvalue).Value.Elements.offset(idx.try_into().unwrap()) };
            let value = unsafe{ (*element).Element.IpAddressOption };
            values.push(value);
            unsafe { DhcpRpcFreeMemory(element as *mut c_void) };
        }

        unsafe { DhcpRpcFreeMemory(optionvalue as *mut c_void) };

        Ok(values)
    }

    fn set_options(&self, optionid: u32, set_values: &[u32]) -> WinDhcpResult<()> {
        if set_values.is_empty() {
            return self.remove_option(optionid).map_err(|e|
                WinDhcpError::new("removing option", e)
            );
        }

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        let mut values = set_values.iter().map( |i|
            DHCP_OPTION_DATA_ELEMENT {
                OptionType: DhcpDWordOption,
                Element: DHCP_OPTION_DATA_ELEMENT_0 {
                    IpAddressOption: *i,
                },
            }
        ).collect::<Vec<DHCP_OPTION_DATA_ELEMENT>>();

        let mut optionvalue = DHCP_OPTION_DATA {
            NumElements: values.len() as u32,
            Elements: values.as_mut_ptr(),
        };

        match unsafe {
            DhcpSetOptionValueV5(
                &self.serveripaddress,
                0x00,
                optionid,
                PCWSTR::null(),
                PCWSTR::null(),
                &mut scopeinfo,
                &mut optionvalue,
            )
        } {
            0 => Ok(()),
            e => Err(WinDhcpError::new("setting option", e)),
        }
    }
}

impl SubnetOptions<Ipv4Addr> for Subnet {
    fn get_options(&self, optionid: u32) -> WinDhcpResult<Vec<Ipv4Addr>> {
        let mut optionvalue: *mut DHCP_OPTION_VALUE = ptr::null_mut();

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        match unsafe {
            DhcpGetOptionValueV5(
                &self.serveripaddress,
                0x00,
                optionid,
                PCWSTR::null(),
                PCWSTR::null(),
                &mut scopeinfo,
                &mut optionvalue,
            )
        } {
            0 => (),
            2 => return Ok(Vec::new()),
            e => return Err(WinDhcpError::new("getting option", e)),
        }

        let len = unsafe { (*optionvalue).Value.NumElements };

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

    fn set_options(&self, optionid: u32, set_values: &[Ipv4Addr]) -> WinDhcpResult<()> {
        if set_values.is_empty() {
            return self.remove_option(optionid).map_err(|e|
                WinDhcpError::new("removing option", e)
            );
        }

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        let mut values = set_values.iter().map( |i|
            DHCP_OPTION_DATA_ELEMENT {
                OptionType: DhcpIpAddressOption,
                Element: DHCP_OPTION_DATA_ELEMENT_0 {
                    IpAddressOption: u32::from(*i),
                },
            }
        ).collect::<Vec<DHCP_OPTION_DATA_ELEMENT>>();

        let mut optionvalue = DHCP_OPTION_DATA {
            NumElements: values.len() as u32,
            Elements: values.as_mut_ptr(),
        };

        match unsafe {
            DhcpSetOptionValueV5(
                &self.serveripaddress,
                0x00,
                optionid,
                PCWSTR::null(),
                PCWSTR::null(),
                &mut scopeinfo,
                &mut optionvalue,
            )
        } {
            0 => Ok(()),
            e => Err(WinDhcpError::new("setting option", e)),
        }
    }
}

impl SubnetOption<String> for Subnet {}
impl SubnetOptions<String> for Subnet {
    fn get_options(&self, optionid: u32) -> WinDhcpResult<Vec<String>> {
        let mut optionvalue: *mut DHCP_OPTION_VALUE = ptr::null_mut();

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        match unsafe {
            DhcpGetOptionValueV5(
                &self.serveripaddress,
                0x00,
                optionid,
                PCWSTR::null(),
                PCWSTR::null(),
                &mut scopeinfo,
                &mut optionvalue,
            )
        } {
            0 => (),
            2 => return Ok(Vec::new()),
            e => return Err(WinDhcpError::new("getting option", e)),
        }

        let len = unsafe { (*optionvalue).Value.NumElements };

        let mut strings = Vec::with_capacity(len as usize);

        for idx in 0..len {
            let element = unsafe{ (*optionvalue).Value.Elements.offset(idx.try_into().unwrap()) };
            let value = unsafe{ (*element).Element.StringDataOption.to_string().unwrap_or_default() }.clone();
            strings.push(value);
            unsafe { DhcpRpcFreeMemory(element as *mut c_void) };
        }

        unsafe { DhcpRpcFreeMemory(optionvalue as *mut c_void) };

        Ok(strings)
    }

    fn set_options(&self, optionid: u32, set_values: &[String]) -> WinDhcpResult<()> {
        if set_values.is_empty() {
            return self.remove_option(optionid).map_err(|e|
                WinDhcpError::new("removing option", e)
            );
        }

        let mut scopeinfo = DHCP_OPTION_SCOPE_INFO {
            ScopeType: DhcpSubnetOptions,
            ScopeInfo: DHCP_OPTION_SCOPE_INFO_0 { SubnetScopeInfo: self.subnetaddress },
        };

        let mut set_values_u16 = set_values.iter()
            .map(|s|s.encode_utf16().chain([0u16]).collect::<Vec<u16>>())
            .collect::<Vec<Vec<u16>>>();

        let mut values = set_values_u16.iter_mut().map( |i|
            DHCP_OPTION_DATA_ELEMENT {
                OptionType: DhcpStringDataOption,
                Element: DHCP_OPTION_DATA_ELEMENT_0 {
                    StringDataOption: PWSTR(i.as_mut_ptr()),
                },
            }
        ).collect::<Vec<DHCP_OPTION_DATA_ELEMENT>>();

        let mut optionvalue = DHCP_OPTION_DATA {
            NumElements: values.len() as u32,
            Elements: values.as_mut_ptr(),
        };

        match unsafe {
            DhcpSetOptionValueV5(
                &self.serveripaddress,
                0x00,
                optionid,
                PCWSTR::null(),
                PCWSTR::null(),
                &mut scopeinfo,
                &mut optionvalue,
            )
        } {
            0 => Ok(()),
            e => Err(WinDhcpError::new("setting option", e)),
        }
    }
}
