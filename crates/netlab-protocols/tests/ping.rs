use netlab_network::{Ipv4Address, MacAddress, Topology};
use netlab_protocols::{run_ping, PingConfig};

fn topology_with_link(enabled: bool) -> (Topology, netlab_network::DeviceId) {
    let mut topology = Topology::new();
    let source = topology
        .add_host(
            "PC1",
            "02:00:00:00:00:01"
                .parse::<MacAddress>()
                .expect("valid MAC"),
            "10.0.0.1".parse::<Ipv4Address>().expect("valid IPv4"),
        )
        .expect("source host");
    let target = topology
        .add_host(
            "PC2",
            "02:00:00:00:00:02"
                .parse::<MacAddress>()
                .expect("valid MAC"),
            "10.0.0.2".parse::<Ipv4Address>().expect("valid IPv4"),
        )
        .expect("target host");
    topology.connect(source, target, 10, enabled).expect("link");
    (topology, source)
}

#[test]
fn disabled_link_produces_real_packet_loss() {
    let (topology, source) = topology_with_link(false);
    let result = run_ping(
        &topology,
        source,
        "10.0.0.2".parse().expect("target"),
        PingConfig::default(),
    )
    .expect("simulation result");
    assert_eq!(result.sent, 1);
    assert_eq!(result.received, 0);
    assert_eq!(result.lost, 1);
    assert!(result.events.iter().any(|event| matches!(
        event.event_type,
        netlab_core::EventType::PacketDropped { .. }
    )));
}

#[test]
fn repeated_runs_are_deterministic() {
    let (topology, source) = topology_with_link(true);
    let config = PingConfig {
        count: 2,
        identifier: 9,
    };
    let first = run_ping(
        &topology,
        source,
        "10.0.0.2".parse().expect("target"),
        config,
    )
    .expect("first run");
    let second = run_ping(
        &topology,
        source,
        "10.0.0.2".parse().expect("target"),
        config,
    )
    .expect("second run");
    assert_eq!(first.rtts, second.rtts);
    assert_eq!(first.events, second.events);
    assert_eq!(first.packets, second.packets);
}
