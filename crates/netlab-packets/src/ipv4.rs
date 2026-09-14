//! Cabeçalho IPv4 mínimo usado pela v0.1.
use netlab_network::Ipv4Address;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpProtocol {
    Icmp,
    Other(u8),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ipv4Header {
    pub src: Ipv4Address,
    pub dst: Ipv4Address,
    pub ttl: u8,
    pub protocol: IpProtocol,
}
impl Ipv4Header {
    pub fn new(src: Ipv4Address, dst: Ipv4Address, protocol: IpProtocol) -> Self {
        Self {
            src,
            dst,
            ttl: 64,
            protocol,
        }
    }
}
