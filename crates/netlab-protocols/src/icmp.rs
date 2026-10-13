//! ICMP Echo e uma sessão de ping executada pelo scheduler real.
use netlab_core::{EventType, NetlabError, Simulation, SimulationConfig, SimulationTime};
use netlab_network::{DeviceId, Ipv4Address, LinkId, NetworkError, Topology};
use netlab_packets::{IcmpMessage, IcmpType, Packet, PacketId, PacketStatus, Payload};
use std::collections::{BTreeMap, HashMap};
#[derive(Debug)]
pub enum ProtocolError {
    Network(NetworkError),
    Simulation(NetlabError),
    InvalidPacket(String),
    HostOffline(String),
    TargetNotFound(Ipv4Address),
    NoLink,
    InvalidCount,
}
impl From<NetworkError> for ProtocolError {
    fn from(e: NetworkError) -> Self {
        Self::Network(e)
    }
}
impl From<NetlabError> for ProtocolError {
    fn from(e: NetlabError) -> Self {
        Self::Simulation(e)
    }
}
impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "{e}"),
            Self::Simulation(e) => write!(f, "{e}"),
            Self::InvalidPacket(message) => write!(f, "invalid packet: {message}"),
            Self::HostOffline(n) => write!(f, "host {n} is offline"),
            Self::TargetNotFound(ip) => write!(f, "target {ip} was not found"),
            Self::NoLink => write!(f, "source and target are not connected by a link"),
            Self::InvalidCount => write!(f, "ping count must be greater than zero"),
        }
    }
}
impl std::error::Error for ProtocolError {}
pub struct IcmpHandler;
impl IcmpHandler {
    pub fn echo_reply(
        request: &Packet,
        at: SimulationTime,
        id: PacketId,
        src_mac: netlab_network::MacAddress,
        dst_mac: netlab_network::MacAddress,
    ) -> Result<Packet, ProtocolError> {
        let Payload::Icmp(message) = &request.payload else {
            return Err(ProtocolError::InvalidPacket("packet is not ICMP".into()));
        };
        if message.icmp_type != IcmpType::EchoRequest {
            return Err(ProtocolError::InvalidPacket(
                "packet is not an Echo Request".into(),
            ));
        }
        let mut reply = Packet::new_icmp(
            id,
            at,
            src_mac,
            dst_mac,
            request.ipv4.dst,
            request.ipv4.src,
            IcmpMessage::echo_reply(message.identifier, message.sequence_number),
        );
        if let Payload::Icmp(reply_message) = &mut reply.payload {
            reply_message.data = message.data.clone();
        }
        Ok(reply)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PingConfig {
    pub count: u16,
    pub identifier: u16,
}
impl Default for PingConfig {
    fn default() -> Self {
        Self {
            count: 1,
            identifier: 1,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PingResult {
    pub source: DeviceId,
    pub target: DeviceId,
    pub sent: u16,
    pub received: u16,
    pub lost: u16,
    pub rtts: Vec<u64>,
    pub events: Vec<netlab_core::SimulationEvent>,
    pub packets: Vec<Packet>,
}
impl PingResult {
    pub fn success(&self) -> bool {
        self.received == self.sent && self.lost == 0
    }
    pub fn average_rtt(&self) -> Option<u64> {
        if self.rtts.is_empty() {
            None
        } else {
            Some(self.rtts.iter().sum::<u64>() / self.rtts.len() as u64)
        }
    }
}
pub fn run_ping(
    topology: &Topology,
    source: DeviceId,
    target_ip: Ipv4Address,
    config: PingConfig,
) -> Result<PingResult, ProtocolError> {
    if config.count == 0 {
        return Err(ProtocolError::InvalidCount);
    }
    let source_host = topology
        .host(source)
        .ok_or(NetworkError::DeviceNotFound(source.0))?;
    let target_host = topology
        .host_by_ip(target_ip)
        .ok_or(ProtocolError::TargetNotFound(target_ip))?;
    let target = target_host.id();
    if !source_host.is_online() {
        return Err(ProtocolError::HostOffline(source_host.name().into()));
    }
    if !target_host.is_online() {
        return Err(ProtocolError::HostOffline(target_host.name().into()));
    }
    let mut network = topology.clone();
    let _ = network
        .connected_links(source)
        .into_iter()
        .next()
        .ok_or(ProtocolError::NoLink)?
        .id();
    let source_ip = source_host.ipv4();
    let source_mac = source_host.mac();
    let target_mac = target_host.mac();
    let mut simulation = Simulation::new(SimulationConfig::default());
    let mut packets = BTreeMap::<PacketId, Packet>::new();
    let mut send_times = HashMap::<u16, SimulationTime>::new();
    for sequence in 0..config.count {
        let id = PacketId(u64::from(sequence) + 1);
        let packet = Packet::new_icmp(
            id,
            SimulationTime::ZERO,
            source_mac,
            target_mac,
            source_ip,
            target_ip,
            IcmpMessage::echo_request(config.identifier, sequence + 1),
        );
        packets.insert(id, packet);
        simulation.schedule_at(
            SimulationTime::ZERO,
            EventType::PacketCreated { packet_id: id.0 },
        )?;
    }
    while let Some(event) = simulation.step() {
        let now = event.timestamp;
        match &event.event_type {
            EventType::PacketCreated { packet_id } => {
                if let Some(packet) = packets.get_mut(&PacketId(*packet_id)) {
                    packet.set_status(PacketStatus::Queued);
                    packet.add_trace(now, "queued");
                    let from = if packet.ipv4.src == source_ip {
                        source
                    } else {
                        target
                    };
                    let Some(link_id) = network
                        .connected_links(from)
                        .into_iter()
                        .next()
                        .map(|link| link.id())
                    else {
                        simulation.schedule_at(
                            now,
                            EventType::PacketDropped {
                                packet_id: *packet_id,
                                reason: "device has no connected link".into(),
                            },
                        )?;
                        continue;
                    };
                    simulation.schedule_at(
                        now,
                        EventType::PacketSent {
                            packet_id: *packet_id,
                            from_device: from.0,
                            link_id: link_id.0,
                        },
                    )?;
                    if let Payload::Icmp(m) = &packet.payload {
                        if m.icmp_type == IcmpType::EchoRequest {
                            send_times.insert(m.sequence_number, now);
                        }
                    }
                }
            }
            EventType::PacketSent {
                packet_id,
                link_id: scheduled_link,
                from_device,
            } => {
                if let Some(packet) = packets.get_mut(&PacketId(*packet_id)) {
                    packet.set_status(PacketStatus::Sent);
                    packet.add_trace(now, "sent");
                    simulation.schedule_at(
                        now,
                        EventType::PacketTransmitting {
                            packet_id: *packet_id,
                            link_id: *scheduled_link,
                            from_device: *from_device,
                        },
                    )?;
                }
            }
            EventType::PacketTransmitting {
                packet_id,
                link_id: scheduled_link,
                from_device,
            } => {
                if let Some(packet) = packets.get_mut(&PacketId(*packet_id)) {
                    packet.set_status(PacketStatus::Transmitting);
                    packet.add_trace(now, "transmitting");
                    let from = DeviceId(*from_device);
                    let link = LinkId(*scheduled_link);
                    let at = network.peer(link, from);
                    let valid = network.link(link).is_some_and(|l| l.enabled())
                        && network.device_is_online(from)
                        && at.is_some_and(|device| network.device_is_online(device));
                    if valid {
                        let delay = network
                            .link(link)
                            .map(|link| link.latency_ms())
                            .unwrap_or_default();
                        simulation.schedule_at(
                            now + delay,
                            EventType::PacketReceived {
                                packet_id: *packet_id,
                                at_device: at.expect("validated peer").0,
                                ingress_link: link.0,
                            },
                        )?;
                    } else {
                        simulation.schedule_at(
                            now,
                            EventType::PacketDropped {
                                packet_id: *packet_id,
                                reason: "link disabled or host offline".into(),
                            },
                        )?;
                    }
                }
            }
            EventType::PacketReceived {
                packet_id,
                at_device,
                ingress_link,
            } => {
                let packet_id_value = *packet_id;
                let at = DeviceId(*at_device);
                if network.is_switch(at) {
                    let Some(ingress_interface) =
                        network.interface_for_device_on_link(at, LinkId(*ingress_link))
                    else {
                        simulation.schedule_at(
                            now,
                            EventType::PacketDropped {
                                packet_id: packet_id_value,
                                reason: "packet arrived on an unknown switch port".into(),
                            },
                        )?;
                        continue;
                    };
                    let (source_mac, destination_mac) = packets
                        .get(&PacketId(packet_id_value))
                        .map(|packet| (packet.ethernet.src, packet.ethernet.dst))
                        .ok_or_else(|| {
                            ProtocolError::InvalidPacket("missing packet at switch".into())
                        })?;
                    let egress_ports = network
                        .switch_mut(at)
                        .ok_or(NetworkError::DeviceNotFound(at.0))?
                        .forward_ports(source_mac, destination_mac, ingress_interface);
                    if let Some(packet) = packets.get_mut(&PacketId(packet_id_value)) {
                        packet.set_status(PacketStatus::Forwarded);
                        packet.add_trace(now, format!("forwarded by switch {at_device}"));
                    }
                    for port in egress_ports {
                        let Some(link) = network.link_for_interface(port).map(|link| link.id())
                        else {
                            continue;
                        };
                        simulation.schedule_at(
                            now,
                            EventType::PacketSent {
                                packet_id: packet_id_value,
                                from_device: at.0,
                                link_id: link.0,
                            },
                        )?;
                    }
                    continue;
                }

                let is_destination = network.host(at).is_some_and(|host| {
                    host.ipv4() == packets[&PacketId(packet_id_value)].ipv4.dst
                });
                let is_request = packets
                    .get(&PacketId(packet_id_value))
                    .is_some_and(|packet| {
                        is_destination
                            && matches!(
                                &packet.payload,
                                Payload::Icmp(message) if message.icmp_type == IcmpType::EchoRequest
                            )
                    });
                if let Some(packet) = packets.get_mut(&PacketId(packet_id_value)) {
                    if is_destination {
                        packet.set_status(PacketStatus::Received);
                        packet.add_trace(now, format!("received by device {at_device}"));
                    } else {
                        packet.set_status(PacketStatus::Dropped);
                        packet.add_trace(now, "ignored by non-destination host");
                    }
                }
                if is_request {
                    let reply_id = PacketId(packets.keys().map(|id| id.0).max().unwrap_or(0) + 1);
                    let request =
                        packets
                            .get(&PacketId(packet_id_value))
                            .ok_or(ProtocolError::Network(NetworkError::InvalidAddress(
                                "missing request".into(),
                            )))?;
                    let reply = IcmpHandler::echo_reply(
                        request,
                        now,
                        reply_id,
                        network
                            .host(at)
                            .map(|host| host.mac())
                            .unwrap_or(target_mac),
                        source_mac,
                    )?;
                    packets.insert(reply_id, reply);
                    simulation.schedule_at(
                        now,
                        EventType::PacketCreated {
                            packet_id: reply_id.0,
                        },
                    )?;
                }
            }
            EventType::PacketDropped { packet_id, .. } => {
                if let Some(packet) = packets.get_mut(&PacketId(*packet_id)) {
                    packet.set_status(PacketStatus::Dropped);
                    packet.add_trace(now, "dropped");
                }
            }
            EventType::DeviceOnline { .. } | EventType::DeviceOffline { .. } => {}
        }
    }
    let mut rtts = Vec::new();
    for packet in packets.values() {
        if let Payload::Icmp(m) = &packet.payload {
            if m.icmp_type == IcmpType::EchoReply && packet.status == PacketStatus::Received {
                if let Some(start) = send_times.get(&m.sequence_number) {
                    let arrival = packet
                        .trace
                        .last()
                        .map(|t| t.timestamp)
                        .unwrap_or(SimulationTime::ZERO);
                    rtts.push(arrival - *start)
                }
            }
        }
    }
    let sent = config.count;
    let received = u16::try_from(rtts.len()).unwrap_or(u16::MAX);
    Ok(PingResult {
        source,
        target,
        sent,
        received,
        lost: sent.saturating_sub(received),
        rtts,
        events: simulation.events().to_vec(),
        packets: packets.into_values().collect(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use netlab_network::{MacAddress, Topology};
    #[test]
    fn ping_uses_latency() {
        let mut t = Topology::new();
        let a = t
            .add_host(
                "PC1",
                "02:00:00:00:00:01".parse::<MacAddress>().expect("mac"),
                "10.0.0.1".parse().expect("ip"),
            )
            .expect("host");
        let b = t
            .add_host(
                "PC2",
                "02:00:00:00:00:02".parse::<MacAddress>().expect("mac"),
                "10.0.0.2".parse().expect("ip"),
            )
            .expect("host");
        t.connect_hosts(a, b, 10).expect("link");
        let r = run_ping(
            &t,
            a,
            "10.0.0.2".parse().expect("ip"),
            PingConfig {
                count: 2,
                identifier: 7,
            },
        )
        .expect("ping");
        assert_eq!(r.rtts, vec![20, 20]);
        assert_eq!(r.received, 2);
    }

    #[test]
    fn ping_crosses_a_learning_switch() {
        let mut topology = Topology::new();
        let a = topology
            .add_host(
                "PC1",
                "02:00:00:00:00:01".parse::<MacAddress>().expect("mac"),
                "10.0.0.1".parse().expect("ip"),
            )
            .expect("host");
        let b = topology
            .add_host(
                "PC2",
                "02:00:00:00:00:02".parse::<MacAddress>().expect("mac"),
                "10.0.0.2".parse().expect("ip"),
            )
            .expect("host");
        let switch = topology.add_switch("SW1", 2).expect("switch");
        topology.connect(a, switch, 5, true).expect("first link");
        topology.connect(b, switch, 7, true).expect("second link");
        let result = run_ping(
            &topology,
            a,
            "10.0.0.2".parse().expect("ip"),
            PingConfig::default(),
        )
        .expect("ping");
        assert_eq!(result.rtts, vec![24]);
        assert!(result.events.iter().any(|event| {
            matches!(
                event.event_type,
                EventType::PacketReceived { at_device, .. } if at_device == switch.0
            )
        }));
    }
}
