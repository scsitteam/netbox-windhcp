use std::{net::Ipv4Addr};

use windows::Win32::NetworkManagement::Dhcp::DHCP_IP_RESERVATION_V4;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ReservationClientTypes {
    Dhcp = 1,
    Bootp = 2,
    Both = 3
}

impl From<u8> for ReservationClientTypes {
    fn from(value: u8) -> Self {
        match value {
            1 => ReservationClientTypes::Dhcp,
            2 => ReservationClientTypes::Bootp,
            _n => ReservationClientTypes::Both,
        }
    }
}

impl From<ReservationClientTypes> for u8 {
    fn from(value: ReservationClientTypes) -> Self {
        match value {
            ReservationClientTypes::Dhcp => 1,
            ReservationClientTypes::Bootp => 2,
            ReservationClientTypes::Both => 3,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Reservation {
    pub ip_address: Ipv4Addr,
    pub for_client: Vec<u8>,
    pub allowed_client_types: ReservationClientTypes,
}

impl From<DHCP_IP_RESERVATION_V4> for Reservation {
    fn from(value: DHCP_IP_RESERVATION_V4) -> Self {
        let len: usize = (unsafe{(*value.ReservedForClient).DataLength}-5).try_into().unwrap();
        Reservation {
            ip_address: Ipv4Addr::from(value.ReservedIpAddress),
            for_client: {
                let mut for_client = Vec::with_capacity(len);
                for idx in 0..len {
                    for_client.insert(idx, unsafe{*(*value.ReservedForClient).Data.offset((5+idx).try_into().unwrap())})
                }
                for_client
            },
            allowed_client_types: value.bAllowedClientTypes.into(),
        }
    }
}