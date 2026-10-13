//! Cabeçalho Ethernet.
use netlab_network::MacAddress;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EtherType {
    Ipv4,
    Other(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EthernetFrame {
    pub src: MacAddress,
    pub dst: MacAddress,
    pub ethertype: EtherType,
}
impl EthernetFrame {
    pub fn new(src: MacAddress, dst: MacAddress) -> Self {
        Self {
            src,
            dst,
            ethertype: EtherType::Ipv4,
        }
    }
}
