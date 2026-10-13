//! Estrutura de pacote e rastreamento do ciclo de vida.
use crate::{EthernetFrame, Ipv4Header, Payload};
use netlab_core::SimulationTime;
use netlab_network::{Ipv4Address, MacAddress};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PacketId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketStatus {
    Created,
    Queued,
    Sent,
    Transmitting,
    Forwarded,
    Received,
    Dropped,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketTrace {
    pub timestamp: SimulationTime,
    pub description: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub id: PacketId,
    pub ethernet: EthernetFrame,
    pub ipv4: Ipv4Header,
    pub payload: Payload,
    pub created_at: SimulationTime,
    pub status: PacketStatus,
    pub trace: Vec<PacketTrace>,
}
impl Packet {
    pub fn new(
        id: PacketId,
        at: SimulationTime,
        ethernet: EthernetFrame,
        ipv4: Ipv4Header,
        payload: Payload,
    ) -> Self {
        Self {
            id,
            ethernet,
            ipv4,
            payload,
            created_at: at,
            status: PacketStatus::Created,
            trace: Vec::new(),
        }
    }
    pub fn new_icmp(
        id: PacketId,
        at: SimulationTime,
        src_mac: MacAddress,
        dst_mac: MacAddress,
        src_ip: Ipv4Address,
        dst_ip: Ipv4Address,
        message: crate::IcmpMessage,
    ) -> Self {
        Self::new(
            id,
            at,
            EthernetFrame::new(src_mac, dst_mac),
            Ipv4Header::new(src_ip, dst_ip, crate::IpProtocol::Icmp),
            Payload::Icmp(message),
        )
    }
    pub fn set_status(&mut self, s: PacketStatus) {
        self.status = s
    }
    pub fn add_trace(&mut self, at: SimulationTime, d: impl Into<String>) {
        self.trace.push(PacketTrace {
            timestamp: at,
            description: d.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Packet, PacketId, PacketStatus};
    use crate::IcmpMessage;
    use netlab_core::SimulationTime;
    use netlab_network::{Ipv4Address, MacAddress};

    #[test]
    fn creates_a_structured_icmp_packet() {
        let packet = Packet::new_icmp(
            PacketId(1),
            SimulationTime::ZERO,
            "02:00:00:00:00:01".parse::<MacAddress>().expect("MAC"),
            "02:00:00:00:00:02".parse::<MacAddress>().expect("MAC"),
            "10.0.0.1".parse::<Ipv4Address>().expect("IPv4"),
            "10.0.0.2".parse::<Ipv4Address>().expect("IPv4"),
            IcmpMessage::echo_request(1, 1),
        );
        assert_eq!(packet.status, PacketStatus::Created);
        assert_eq!(packet.ipv4.ttl, 64);
        assert_eq!(packet.id, PacketId(1));
    }
}
