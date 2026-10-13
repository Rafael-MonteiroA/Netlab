//! Endereço MAC estruturado em 48 bits.
use std::{fmt, str::FromStr};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MacAddress([u8; 6]);
impl MacAddress {
    pub const fn new(o: [u8; 6]) -> Self {
        Self(o)
    }
    pub const fn octets(self) -> [u8; 6] {
        self.0
    }
    pub const fn broadcast() -> Self {
        Self([0xff; 6])
    }
    pub fn parse(v: &str) -> Result<Self, MacAddressParseError> {
        v.parse()
    }
}
impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MacAddressParseError;
impl fmt::Display for MacAddressParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MAC must contain six hexadecimal octets")
    }
}
impl std::error::Error for MacAddressParseError {}
impl FromStr for MacAddress {
    type Err = MacAddressParseError;
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        let p: Vec<_> = v.split(':').collect();
        if p.len() != 6 || p.iter().any(|x| x.len() != 2) {
            return Err(MacAddressParseError);
        }
        let mut o = [0; 6];
        for (i, x) in p.iter().enumerate() {
            o[i] = u8::from_str_radix(x, 16).map_err(|_| MacAddressParseError)?
        }
        Ok(Self(o))
    }
}
#[cfg(test)]
mod tests {
    use super::MacAddress;
    #[test]
    fn parses() {
        assert_eq!(
            "02:00:00:00:00:01"
                .parse::<MacAddress>()
                .expect("valid")
                .to_string(),
            "02:00:00:00:00:01"
        )
    }
    #[test]
    fn rejects() {
        assert!("02:00:00".parse::<MacAddress>().is_err())
    }
}
