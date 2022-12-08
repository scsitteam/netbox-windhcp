use std::net::Ipv4Addr;
use std::os::raw::c_void;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use log::debug;
use windows::core::{HSTRING, PWSTR};
use windows::Win32::NetworkManagement::Dhcp::*;

static GLOBAL_DHCP_INIT_COUNT: AtomicUsize = AtomicUsize::new(0);

pub mod subnet;
pub use subnet::*;
pub mod error;
pub use error::*;

#[derive(Debug, PartialEq)]
pub struct WinDhcp {
    serveripaddress: HSTRING,
}

impl WinDhcp {
    pub fn new(server: &str) -> Self {
        let serveripaddress: HSTRING = HSTRING::from(server);
        if GLOBAL_DHCP_INIT_COUNT.fetch_add(1, Ordering::SeqCst) == 0 {
            let mut version: u32 = 0;
            let ret = unsafe { DhcpCApiInitialize(&mut version) };
            
            debug!("Init WinDhcp Api v{} [{}]", version, ret);
        }

        WinDhcp { serveripaddress }
    }

    pub fn get_version(&self) -> WinDhcpResult<(u32, u32)> {
        let mut major: u32 = 0;
        let mut minor: u32 = 0;

        match unsafe { DhcpGetVersion(&self.serveripaddress, &mut major, &mut minor) } {
            0 => Ok((major, minor)),
            e => Err(WinDhcpError::new("getting version", e)),
        }
    }

    pub fn get_subnets(&self) -> WinDhcpResult<Vec<Ipv4Addr>> {
        let mut resumehandle: u32 = 0;
        let mut elementsread: u32 = 0;
        let mut elementstotal: u32 = 0;

        let mut enuminfo: *mut DHCP_IP_ARRAY = ptr::null_mut();

        match unsafe { DhcpEnumSubnets(&self.serveripaddress,
            &mut resumehandle,
            0xFFFFFFFF,
            &mut enuminfo,
            &mut elementsread,
            &mut elementstotal)} {
                0 => (),
                //ERROR_NO_MORE_ITEMS
                259 => { return Ok(Vec::new()); },
                e => { return Err(WinDhcpError::new("listing subnets", e)); },
        }

        let data: DHCP_IP_ARRAY = unsafe {*enuminfo};

        let mut subnets = Vec::with_capacity(data.NumElements.try_into().unwrap());

        for idx in 0..data.NumElements {
            subnets.push(Ipv4Addr::from(unsafe { *data.Elements.offset(idx.try_into().unwrap()) }));
        }

        unsafe { DhcpRpcFreeMemory((*enuminfo).Elements as *mut c_void) };
        unsafe { DhcpRpcFreeMemory(enuminfo as *mut c_void) };

        Ok(subnets)
    }

    pub fn get_or_create_subnet(&self, subnetaddress: &Ipv4Addr, subnetmask: &Ipv4Addr) -> Result<Subnet, u32> {
        match Subnet::get(&self.serveripaddress, subnetaddress)? {
            Some(subnet) => Ok(subnet),
            None => Subnet::create(&self.serveripaddress, subnetaddress, subnetmask),
        }
    }

    pub fn get_client_name(&self, clientip: Ipv4Addr) -> Result<String, u32> {
        let mut clientinfo: *mut DHCP_CLIENT_INFO_V4 = ptr::null_mut();

        let searchinfo = DHCP_SEARCH_INFO {
            SearchType: DHCP_SEARCH_INFO_TYPE(0),
            SearchInfo: DHCP_SEARCH_INFO_0 { ClientIpAddress: u32::from(clientip) },
        };
        match unsafe { DhcpGetClientInfoV4(&self.serveripaddress, &searchinfo, &mut clientinfo)} {
                0 => (),
                n => { return Err(n); },
        }

        let info = unsafe{ *clientinfo };

        let name = match info.ClientName.is_null() {
            true => String::from(""),
            false => unsafe{info.ClientName.to_string()}.unwrap_or_default(),
        };

        unsafe { DhcpRpcFreeMemory((*clientinfo).ClientName.as_ptr() as *mut c_void) };
        unsafe { DhcpRpcFreeMemory((*clientinfo).ClientComment.as_ptr() as *mut c_void) };
        unsafe { DhcpRpcFreeMemory((*clientinfo).ClientHardwareAddress.Data as *mut c_void) };
        unsafe { DhcpRpcFreeMemory((*clientinfo).OwnerHost.HostName.as_ptr() as *mut c_void) };
        unsafe { DhcpRpcFreeMemory((*clientinfo).OwnerHost.NetBiosName.as_ptr() as *mut c_void) };
        unsafe { DhcpRpcFreeMemory(clientinfo as *mut c_void) };

        Ok(name)
    }
    
    pub fn set_client_name(&self, clientip: Ipv4Addr, name: &str) -> WinDhcpResult<()>  {
        let mut clientinfo: *mut DHCP_CLIENT_INFO_V4 = ptr::null_mut();

        let searchinfo = DHCP_SEARCH_INFO {
            SearchType: DHCP_SEARCH_INFO_TYPE(0),
            SearchInfo: DHCP_SEARCH_INFO_0 { ClientIpAddress: u32::from(clientip) },
        };
        match unsafe { DhcpGetClientInfoV4(&self.serveripaddress, &searchinfo, &mut clientinfo)} {
                0 => (),
                e => return Err(WinDhcpError::new("setting client name", e)),
        }

        let mut info = unsafe{ *clientinfo };
        info.ClientName = PWSTR::from_raw(name.encode_utf16().chain(::std::iter::once(0)).collect::<Vec<u16>>().as_mut_ptr());
    
        let ret = match unsafe { DhcpSetClientInfoV4(&self.serveripaddress, &info)} {
            0 => Ok(()),
            e => Err(WinDhcpError::new("setting client name", e)),
        };

        unsafe { DhcpRpcFreeMemory((*clientinfo).ClientName.as_ptr() as *mut c_void) };
        unsafe { DhcpRpcFreeMemory((*clientinfo).ClientComment.as_ptr() as *mut c_void) };
        unsafe { DhcpRpcFreeMemory((*clientinfo).ClientHardwareAddress.Data as *mut c_void) };
        unsafe { DhcpRpcFreeMemory((*clientinfo).OwnerHost.HostName.as_ptr() as *mut c_void) };
        unsafe { DhcpRpcFreeMemory((*clientinfo).OwnerHost.NetBiosName.as_ptr() as *mut c_void) };
        unsafe { DhcpRpcFreeMemory(clientinfo as *mut c_void) };

        ret
    }

}

impl Drop for WinDhcp {
    fn drop(&mut self) {
        if GLOBAL_DHCP_INIT_COUNT.fetch_sub(1, Ordering::SeqCst) == 1 {
            unsafe { DhcpCApiCleanup() };
            debug!("DhcpCApiCleanup");
        }
    }
}