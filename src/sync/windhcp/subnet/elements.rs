use std::{os::raw::c_void, ptr};

use windows::Win32::NetworkManagement::Dhcp::{DhcpRpcFreeMemory, DHCP_BOOTP_IP_RANGE, DHCP_SUBNET_ELEMENT_INFO_ARRAY_V5, DhcpEnumSubnetElementsV5, DhcpIpRangesDhcpBootp, DHCP_SUBNET_ELEMENT_DATA_V5, DHCP_SUBNET_ELEMENT_DATA_V5_0, DhcpAddSubnetElementV5, DhcpReservedIps, DHCP_IP_RESERVATION_V4, DHCP_BINARY_DATA, DhcpRemoveSubnetElementV5, DhcpFullForce};

use super::Subnet;
use super::reservation::Reservation;

pub trait SubnetElements<T> {
    fn get_first_element(&self) -> Result<Option<T>, u32> {
        todo!()
    }
    fn get_elements(&self) -> Result<Vec<T>, u32> {
        todo!()
    }
    fn add_element(&self, _element: &mut T) -> Result<(), u32> {
        todo!()
    }
    fn remove_element(&self, _element: &mut T) -> Result<(), u32> {
        todo!()
    }
}

impl SubnetElements<DHCP_BOOTP_IP_RANGE> for Subnet {
    fn get_first_element(&self) -> Result<Option<DHCP_BOOTP_IP_RANGE>, u32> {
        let mut resumehandle: u32 = 0;
        let mut elementsread: u32 = 0;
        let mut elementstotal: u32 = 0;

        let mut enumelementinfo: *mut DHCP_SUBNET_ELEMENT_INFO_ARRAY_V5 = ptr::null_mut();

        match unsafe {
            DhcpEnumSubnetElementsV5(
                &self.serveripaddress,
                self.subnetaddress,
                DhcpIpRangesDhcpBootp,
                &mut resumehandle,
                0xFFFFFFFF,
                &mut enumelementinfo,
                &mut elementsread,
                &mut elementstotal,
            )
        } {
            0 => unsafe {
                let range = DHCP_BOOTP_IP_RANGE { ..(*(*(*enumelementinfo).Elements).Element.IpRange) };

                for idx in 0usize..(*enumelementinfo).NumElements.try_into().unwrap() {
                    DhcpRpcFreeMemory((*(*enumelementinfo).Elements.offset(idx.try_into().unwrap())).Element.IpRange as *mut c_void); 
                }
                DhcpRpcFreeMemory((*enumelementinfo).Elements as *mut c_void); 
                DhcpRpcFreeMemory(enumelementinfo as *mut c_void);

                Ok(Some(range))
            },
            //ERROR_NO_MORE_ITEMS
            259 => {
                Ok(None)
            },
            n => {
                Err(n)
            }
        }
    }

    fn add_element(&self, element: &mut DHCP_BOOTP_IP_RANGE) -> Result<(), u32> {
        let addelementinfo = DHCP_SUBNET_ELEMENT_DATA_V5  {
            ElementType: DhcpIpRangesDhcpBootp,
            Element: DHCP_SUBNET_ELEMENT_DATA_V5_0 {
                IpRange: element,
            },
        };

        match unsafe { DhcpAddSubnetElementV5(&self.serveripaddress, self.subnetaddress, &addelementinfo) } {
            0 => Ok(()),
            n => Err(n),
        }
    }
}


impl SubnetElements<Reservation> for Subnet {
    fn get_elements(&self) -> Result<Vec<Reservation>, u32> {
        let mut resumehandle: u32 = 0;
        let mut elementsread: u32 = 0;
        let mut elementstotal: u32 = 0;

        let mut enumelementinfo: *mut DHCP_SUBNET_ELEMENT_INFO_ARRAY_V5 = ptr::null_mut();

        match unsafe {
            DhcpEnumSubnetElementsV5(
                &self.serveripaddress,
                self.subnetaddress,
                DhcpReservedIps,
                &mut resumehandle,
                0xFFFFFFFF,
                &mut enumelementinfo,
                &mut elementsread,
                &mut elementstotal,
            )
        } {
            0 => {
                let mut elements = Vec::new();

                for idx in 0usize..unsafe{ (*enumelementinfo).NumElements.try_into().unwrap() } {
                    let res = unsafe {*(*(*enumelementinfo).Elements.offset(idx.try_into().unwrap())).Element.ReservedIp};
                    elements.insert(idx, Reservation::from(res));
                }

                unsafe {
                    for idx in 0usize..(*enumelementinfo).NumElements.try_into().unwrap() {
                        DhcpRpcFreeMemory((*(*enumelementinfo).Elements.offset(idx.try_into().unwrap())).Element.ReservedIp as *mut c_void); 
                    }
                    DhcpRpcFreeMemory((*enumelementinfo).Elements as *mut c_void); 
                    DhcpRpcFreeMemory(enumelementinfo as *mut c_void);
                }

                Ok(elements)
            },
            //ERROR_NO_MORE_ITEMS
            259 => {
                Ok(vec![])
            }
            n => {
                Err(n)
            }
        }
    }

    fn add_element(&self, element: &mut Reservation) -> Result<(), u32> {
        
        let mut for_client = DHCP_BINARY_DATA {
            DataLength: element.for_client.len().try_into().unwrap(),
            Data: element.for_client[..].as_mut_ptr()
        };

        let mut reserved_ip = DHCP_IP_RESERVATION_V4 {
            ReservedIpAddress: element.ip_address.into(),
            ReservedForClient: &mut for_client,
            bAllowedClientTypes: element.allowed_client_types.clone().into()
        };

        let addelementinfo = DHCP_SUBNET_ELEMENT_DATA_V5  {
            ElementType: DhcpReservedIps,
            Element: DHCP_SUBNET_ELEMENT_DATA_V5_0 {
                ReservedIp: &mut reserved_ip,
            },
        };

        match unsafe { DhcpAddSubnetElementV5(&self.serveripaddress, self.subnetaddress, &addelementinfo) } {
            0 => Ok(()),
            n => Err(n),
        }
    }

    fn remove_element(&self, element: &mut Reservation) -> Result<(), u32> {
        
        let mut for_client = DHCP_BINARY_DATA {
            DataLength: element.for_client.len().try_into().unwrap(),
            Data: element.for_client[..].as_mut_ptr()
        };

        let mut reserved_ip = DHCP_IP_RESERVATION_V4 {
            ReservedIpAddress: element.ip_address.into(),
            ReservedForClient: &mut for_client,
            bAllowedClientTypes: element.allowed_client_types.clone().into()
        };

        let removeelementinfo = DHCP_SUBNET_ELEMENT_DATA_V5  {
            ElementType: DhcpReservedIps,
            Element: DHCP_SUBNET_ELEMENT_DATA_V5_0 {
                ReservedIp: &mut reserved_ip,
            },
        };

        match unsafe { DhcpRemoveSubnetElementV5(&self.serveripaddress, self.subnetaddress, &removeelementinfo, DhcpFullForce) } {
            0 => Ok(()),
            n => Err(n),
        }
    }
}