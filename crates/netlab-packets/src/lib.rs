//! Representações estruturadas dos pacotes da simulação.
pub mod ethernet;
pub mod icmp;
pub mod ipv4;
pub mod packet;
pub mod payload;
pub use ethernet::{EtherType, EthernetFrame};
pub use icmp::{IcmpMessage, IcmpType};
pub use ipv4::{IpProtocol, Ipv4Header};
pub use packet::{Packet, PacketId, PacketStatus, PacketTrace};
pub use payload::Payload;
