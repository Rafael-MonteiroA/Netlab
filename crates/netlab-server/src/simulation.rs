use axum::{http::StatusCode, Json};
use netlab_network::{Ipv4Address, MacAddress, Topology};
use netlab_protocols::{run_ping, PingConfig};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;

#[derive(Deserialize)]
pub struct Scenario {
    hosts: Vec<Host>,
    source: usize,
    target: usize,
    count: u16,
}
#[derive(Deserialize)]
struct Host {
    name: String,
    ip: String,
    latency_ms: u64,
    enabled: bool,
}

pub async fn simulate(
    Json(scenario): Json<Scenario>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // The bounded star topology cannot introduce switching loops or unbounded event growth.
    execute(scenario)
        .map(Json)
        .map_err(|message| (StatusCode::BAD_REQUEST, Json(json!({"error":message}))))
}

fn execute(s: Scenario) -> Result<Value, String> {
    if !(2..=16).contains(&s.hosts.len()) || !(1..=10).contains(&s.count) {
        return Err("Use 2 a 16 hosts e 1 a 10 pings.".into());
    }
    if s.source >= s.hosts.len() || s.target >= s.hosts.len() || s.source == s.target {
        return Err("Selecione origem e destino diferentes.".into());
    }
    let mut topology = Topology::new();
    let switch = topology
        .add_switch("SW1", s.hosts.len())
        .map_err(|e| e.to_string())?;
    let mut ids = Vec::new();
    let mut ips = HashSet::new();
    let mut names = HashSet::new();
    for (index, host) in s.hosts.iter().enumerate() {
        if host.name.trim().is_empty() || host.name.chars().count() > 40 || host.latency_ms > 1000 {
            return Err(
                "Nome obrigatório (até 40 caracteres) e latência entre 0 e 1000 ms.".into(),
            );
        }
        if !names.insert(host.name.trim()) {
            return Err("Cada host precisa de um nome diferente.".into());
        }
        let ip: Ipv4Address = host.ip.parse().map_err(|_| "Endereço IPv4 inválido.")?;
        if !ips.insert(ip) {
            return Err("Cada host precisa de um IP diferente.".into());
        }
        let mac = MacAddress::new([2, 0, 0, 0, 0, (index + 1) as u8]);
        let id = topology
            .add_host(&host.name, mac, ip)
            .map_err(|e| e.to_string())?;
        topology
            .connect(id, switch, host.latency_ms, host.enabled)
            .map_err(|e| e.to_string())?;
        ids.push(id);
    }
    let result = run_ping(
        &topology,
        ids[s.source],
        s.hosts[s.target].ip.parse().map_err(|_| "IPv4 inválido")?,
        PingConfig {
            count: s.count,
            identifier: 1,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(json!({
        "sent":result.sent, "received":result.received, "lost":result.lost,
        "average_rtt":result.average_rtt(), "rtts":result.rtts,
        "events":result.events.iter().map(|event| json!({"id":event.id.0, "time_ms":event.timestamp.as_millis(), "detail":format!("{:?}", event.event_type)})).collect::<Vec<_>>(),
        "packets":result.packets.iter().map(|p| json!({"id":p.id.0, "source":p.ipv4.src.to_string(), "target":p.ipv4.dst.to_string(), "status":format!("{:?}", p.status), "trace":p.trace.iter().map(|t| json!({"time_ms":t.timestamp.as_millis(), "detail":t.description})).collect::<Vec<_>>()})).collect::<Vec<_>>()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scenario(enabled: bool) -> Scenario {
        Scenario {
            hosts: vec![
                Host {
                    name: "PC1".into(),
                    ip: "10.0.0.1".into(),
                    latency_ms: 5,
                    enabled: true,
                },
                Host {
                    name: "PC2".into(),
                    ip: "10.0.0.2".into(),
                    latency_ms: 7,
                    enabled,
                },
            ],
            source: 0,
            target: 1,
            count: 3,
        }
    }
    #[test]
    fn runs_existing_engine() {
        let output = execute(scenario(true)).unwrap();
        assert_eq!(output["average_rtt"], 24);
        assert_eq!(output["received"], 3);
        assert!(!output["packets"].as_array().unwrap().is_empty());
    }
    #[test]
    fn disabled_link_loses_packets() {
        let output = execute(scenario(false)).unwrap();
        assert_eq!(output["lost"], 3);
    }
    #[test]
    fn rejects_invalid_topology() {
        let mut s = scenario(true);
        s.target = 99;
        assert!(execute(s).is_err());
        let mut s = scenario(true);
        s.hosts[1].ip = s.hosts[0].ip.clone();
        assert!(execute(s).is_err());
        let mut s = scenario(true);
        s.count = 0;
        assert!(execute(s).is_err());
    }
}
