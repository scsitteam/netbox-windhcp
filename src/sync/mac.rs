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
        mac.split(':')
            .map(|h| T::from_str_radix(h, 16).unwrap_or_default())
            .collect::<Vec<T>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_ips() {
        let mac = Vec::<u8>::from_mac("00:11");
        assert_eq!(mac, vec!(0x00, 0x11))
    }

    #[test]
    fn it_builds_mac() {
        let mac = vec![0x22, 0x33];
        assert_eq!(mac.as_mac(), "22:33")
    }
}
