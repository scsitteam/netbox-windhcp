pub trait MacAddr<T> {
    fn as_mac(&self) -> String;
    fn from_mac(mac: &str) -> Vec<T>;
}
impl<T: std::fmt::UpperHex + num::Num + Default> MacAddr<T> for Vec<T> {
    fn as_mac(&self) -> String {
        self.iter()
            .map(|d| format!("{:02X}", d)).collect::<Vec<String>>()
            .join(":")
    }

    fn from_mac(mac: &str) -> Vec<T> {
        regex::Regex::new(r"[0-9A-Fa-f]{2}").unwrap().captures_iter(mac)
            .map(|h| T::from_str_radix(&h[0], 16).unwrap_or_default())
            .collect::<Vec<T>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_mac() {
        let mac = Vec::<u8>::from_mac("00:aa:bB:CC");
        assert_eq!(mac, vec!(0x00, 0xaa, 0xbb, 0xcc))
    }

    #[test]
    fn it_parses_mac_w_colon() {
        let mac = Vec::<u8>::from_mac("00:11:22:33");
        assert_eq!(mac, vec!(0x00, 0x11, 0x22, 0x33))
    }

    #[test]
    fn it_parses_mac_w_dash() {
        let mac = Vec::<u8>::from_mac("00-11-2233");
        assert_eq!(mac, vec!(0x00, 0x11, 0x22, 0x33))
    }

    #[test]
    fn it_parses_mac_wo_separation() {
        let mac = Vec::<u8>::from_mac("00112233");
        assert_eq!(mac, vec!(0x00, 0x11, 0x22, 0x33))
    }

    #[test]
    fn it_builds_mac() {
        let mac = vec![0x22, 0x33];
        assert_eq!(mac.as_mac(), "22:33")
    }
}
