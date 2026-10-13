//! Rede lógica com prefixo CIDR simples.
use crate::{Ipv4Address, NetworkError};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Network {
    name: String,
    address: Ipv4Address,
    prefix_len: u8,
}
impl Network {
    pub fn new(name: impl Into<String>, address: Ipv4Address, p: u8) -> Result<Self, NetworkError> {
        if p > 32 {
            return Err(NetworkError::InvalidAddress(format!("prefix length {p}")));
        }
        Ok(Self {
            name: name.into(),
            address,
            prefix_len: p,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn address(&self) -> Ipv4Address {
        self.address
    }
    pub fn prefix_len(&self) -> u8 {
        self.prefix_len
    }
    pub fn contains(&self, a: Ipv4Address) -> bool {
        self.address.is_same_subnet(a, self.prefix_len)
    }
}
