//! Endereço IPv4 estruturado em quatro octetos.
use std::{fmt, str::FromStr};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ipv4Address([u8; 4]);
impl Ipv4Address {
    pub const fn new(o: [u8; 4]) -> Self {
        Self(o)
    }
    pub const fn octets(self) -> [u8; 4] {
        self.0
    }
    pub fn parse(v: &str) -> Result<Self, Ipv4AddressParseError> {
        v.parse()
    }
    pub fn is_same_subnet(self, o: Self, p: u8) -> bool {
        if p > 32 {
            return false;
        }
        let m = if p == 0 { 0 } else { u32::MAX << (32 - p) };
        (u32::from(self) & m) == (u32::from(o) & m)
    }
}
impl From<[u8; 4]> for Ipv4Address {
    fn from(v: [u8; 4]) -> Self {
        Self::new(v)
    }
}
impl From<Ipv4Address> for u32 {
    fn from(v: Ipv4Address) -> Self {
        u32::from_be_bytes(v.0)
    }
}
impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ipv4AddressParseError;
impl fmt::Display for Ipv4AddressParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IPv4 must contain four decimal octets")
    }
}
impl std::error::Error for Ipv4AddressParseError {}
impl FromStr for Ipv4Address {
    type Err = Ipv4AddressParseError;
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        let p: Vec<_> = v.split('.').collect();
        if p.len() != 4 {
            return Err(Ipv4AddressParseError);
        }
        let mut o = [0; 4];
        for (i, x) in p.iter().enumerate() {
            o[i] = x.parse::<u8>().map_err(|_| Ipv4AddressParseError)?
        }
        Ok(Self(o))
    }
}
#[cfg(test)]
mod tests {
    use super::Ipv4Address;
    #[test]
    fn parses() {
        let a: Ipv4Address = "10.0.0.1".parse().expect("valid");
        assert_eq!(a.to_string(), "10.0.0.1");
        assert!(a.is_same_subnet("10.0.0.2".parse().expect("valid"), 24))
    }
}
