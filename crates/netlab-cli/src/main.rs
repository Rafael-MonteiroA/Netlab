use netlab_network::{DeviceId, Ipv4Address, MacAddress, Topology};
use netlab_packets::{IcmpType, Packet, Payload};
use netlab_protocols::{run_ping, PingConfig, PingResult, ProtocolError};
use std::{env, fs, io, path::Path};

const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Version,
    Run(String),
    Ping {
        scenario: String,
        host: String,
        target: String,
        count: u16,
    },
    Inspect {
        scenario: String,
        id: u64,
    },
}
fn parse_command(args: &[String]) -> Result<Command, String> {
    match args.first().map(String::as_str) {
        None | Some("--help") | Some("-h") | Some("help") => Ok(Command::Help),
        Some("version") | Some("--version") | Some("-V") => Ok(Command::Version),
        Some("run") => args
            .get(1)
            .cloned()
            .map(Command::Run)
            .ok_or_else(|| "run requires a scenario path".into()),
        Some("ping") => {
            if args.len() < 4 {
                return Err("ping requires <scenario> <host> <target-ip> [count]".into());
            }
            let count = args
                .get(4)
                .map(|v| {
                    v.parse::<u16>()
                        .map_err(|_| "count must be an integer".to_string())
                })
                .transpose()?
                .unwrap_or(1);
            Ok(Command::Ping {
                scenario: args[1].clone(),
                host: args[2].clone(),
                target: args[3].clone(),
                count,
            })
        }
        Some("inspect") => {
            if args.len() != 4 || args[1] != "packet" {
                return Err("inspect usage: inspect packet <id> <scenario>".into());
            }
            Ok(Command::Inspect {
                scenario: args[3].clone(),
                id: args[2]
                    .parse()
                    .map_err(|_| "packet id must be an integer")?,
            })
        }
        Some(value) => Err(format!("unknown command '{value}'")),
    }
}
fn print_help() {
    println!("NetLab\n\nNetwork Simulation & Virtual Internet Laboratory\n\nUSAGE:\n    netlab <COMMAND>\n\nCOMMANDS:\n    version                              Show the NetLab version\n    run <scenario>                      Execute a scenario\n    ping <scenario> <host> <target-ip>  Execute an ICMP ping\n    inspect packet <id> <scenario>      Inspect a packet after execution\n    help                                 Show this help message");
}

#[derive(Debug)]
struct HostSpec {
    name: String,
    mac: String,
    ip: String,
}
#[derive(Debug)]
struct LinkSpec {
    from: String,
    to: String,
    latency_ms: u64,
    enabled: bool,
}
#[derive(Debug)]
struct SwitchSpec {
    name: String,
    ports: usize,
}
#[derive(Debug)]
struct ActionSpec {
    from: String,
    target: String,
    count: u16,
}
#[derive(Debug)]
struct Scenario {
    name: String,
    version: String,
    hosts: Vec<HostSpec>,
    switches: Vec<SwitchSpec>,
    links: Vec<LinkSpec>,
    action: ActionSpec,
}
#[derive(Debug)]
struct CliError(String);
impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for CliError {}
impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        Self(e.to_string())
    }
}
impl From<ProtocolError> for CliError {
    fn from(e: ProtocolError) -> Self {
        Self(e.to_string())
    }
}
fn value(raw: &str) -> String {
    let v = raw.trim();
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        v[1..v.len() - 1].to_string()
    } else {
        v.to_string()
    }
}
fn parse_scenario(path: &Path) -> Result<Scenario, CliError> {
    let text = fs::read_to_string(path)?;
    let mut name = "Unnamed".to_string();
    let mut version = "0.1".to_string();
    let mut hosts = Vec::new();
    let mut switches = Vec::new();
    let mut links = Vec::new();
    let mut action = ActionSpec {
        from: String::new(),
        target: String::new(),
        count: 1,
    };
    enum Section {
        Root,
        Simulation,
        Host,
        Link,
        Switch,
        Action,
    }
    let mut section = Section::Root;
    for raw_line in text.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line == "[simulation]" {
            section = Section::Simulation;
            continue;
        }
        if line == "[[hosts]]" {
            hosts.push(HostSpec {
                name: String::new(),
                mac: String::new(),
                ip: String::new(),
            });
            section = Section::Host;
            continue;
        }
        if line == "[[links]]" {
            links.push(LinkSpec {
                from: String::new(),
                to: String::new(),
                latency_ms: 0,
                enabled: true,
            });
            section = Section::Link;
            continue;
        }
        if line == "[[switches]]" {
            switches.push(SwitchSpec {
                name: String::new(),
                ports: 2,
            });
            section = Section::Switch;
            continue;
        }
        if line == "[[actions]]" {
            section = Section::Action;
            continue;
        }
        let Some((key, raw)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let parsed = value(raw);
        match section {
            Section::Root => match key {
                "name" => name = parsed,
                "version" => version = parsed,
                _ => {}
            },
            Section::Simulation => {}
            Section::Host => {
                if let Some(host) = hosts.last_mut() {
                    match key {
                        "name" => host.name = parsed,
                        "mac" => host.mac = parsed,
                        "ip" => host.ip = parsed,
                        _ => {}
                    }
                }
            }
            Section::Link => {
                if let Some(link) = links.last_mut() {
                    match key {
                        "from" => link.from = parsed,
                        "to" => link.to = parsed,
                        "latency_ms" => {
                            link.latency_ms = parsed
                                .parse()
                                .map_err(|_| CliError("invalid latency_ms".into()))?
                        }
                        "enabled" => {
                            link.enabled = parsed
                                .parse()
                                .map_err(|_| CliError("enabled must be true or false".into()))?
                        }
                        _ => {}
                    }
                }
            }
            Section::Switch => {
                if let Some(switch) = switches.last_mut() {
                    match key {
                        "name" => switch.name = parsed,
                        "ports" => {
                            switch.ports = parsed
                                .parse()
                                .map_err(|_| CliError("ports must be an integer".into()))?
                        }
                        _ => {}
                    }
                }
            }
            Section::Action => match key {
                "from" => action.from = parsed,
                "target" => action.target = parsed,
                "count" => {
                    action.count = parsed
                        .parse()
                        .map_err(|_| CliError("invalid ping count".into()))?
                }
                _ => {}
            },
        }
    }
    if hosts.len() < 2 {
        return Err(CliError("scenario must define at least two hosts".into()));
    }
    Ok(Scenario {
        name,
        version,
        hosts,
        switches,
        links,
        action,
    })
}
fn build_topology(scenario: &Scenario) -> Result<Topology, CliError> {
    let mut topology = Topology::new();
    for host in &scenario.hosts {
        let mac: MacAddress = host
            .mac
            .parse()
            .map_err(|e| CliError(format!("host {}: invalid MAC ({e})", host.name)))?;
        let ip: Ipv4Address = host
            .ip
            .parse()
            .map_err(|e| CliError(format!("host {}: invalid IPv4 ({e})", host.name)))?;
        topology
            .add_host(&host.name, mac, ip)
            .map_err(|e| CliError(e.to_string()))?;
    }
    for switch in &scenario.switches {
        topology
            .add_switch(&switch.name, switch.ports)
            .map_err(|e| CliError(format!("switch {}: {e}", switch.name)))?;
    }
    for link in &scenario.links {
        let from = topology
            .device_id_by_name(&link.from)
            .ok_or_else(|| CliError(format!("unknown host {}", link.from)))?;
        let to = topology
            .device_id_by_name(&link.to)
            .ok_or_else(|| CliError(format!("unknown host {}", link.to)))?;
        topology
            .connect(from, to, link.latency_ms, link.enabled)
            .map_err(|e| CliError(e.to_string()))?;
    }
    Ok(topology)
}
fn packet(result: &PingResult, id: u64) -> Option<&Packet> {
    result.packets.iter().find(|packet| packet.id.0 == id)
}
fn host_name(topology: &Topology, id: DeviceId) -> String {
    topology
        .device_name(id)
        .map(str::to_string)
        .unwrap_or_else(|| format!("device-{id:?}"))
}
fn print_events(result: &PingResult, topology: &Topology) {
    for event in &result.events {
        let ms = event.timestamp.as_millis();
        match &event.event_type {
            netlab_core::EventType::PacketCreated { packet_id } => {
                if let Some(packet) = packet(result, *packet_id) {
                    match &packet.payload {
                        Payload::Icmp(message) if message.icmp_type == IcmpType::EchoRequest => {
                            println!("[{ms:04}ms] ICMP Echo Request created")
                        }
                        Payload::Icmp(message) if message.icmp_type == IcmpType::EchoReply => {
                            println!("[{ms:04}ms] ICMP Echo Reply created")
                        }
                        _ => println!("[{ms:04}ms] Packet #{packet_id} created"),
                    }
                }
            }
            netlab_core::EventType::PacketSent {
                packet_id,
                from_device,
                link_id,
            } => println!(
                "[{ms:04}ms] Packet #{packet_id} sent by {} via link-{link_id}",
                host_name(topology, DeviceId(*from_device))
            ),
            netlab_core::EventType::PacketReceived {
                packet_id,
                at_device,
                ..
            } => println!(
                "[{ms:04}ms] Packet #{packet_id} received by {}",
                host_name(topology, DeviceId(*at_device))
            ),
            netlab_core::EventType::PacketTransmitting {
                packet_id, link_id, ..
            } => {
                println!("[{ms:04}ms] Packet #{packet_id} transmitting on link-{link_id}")
            }
            netlab_core::EventType::PacketDropped { packet_id, reason } => {
                println!("[{ms:04}ms] Packet #{packet_id} dropped: {reason}")
            }
            netlab_core::EventType::DeviceOnline { device_id } => println!(
                "[{ms:04}ms] {} online",
                host_name(topology, DeviceId(*device_id))
            ),
            netlab_core::EventType::DeviceOffline { device_id } => println!(
                "[{ms:04}ms] {} offline",
                host_name(topology, DeviceId(*device_id))
            ),
        }
    }
}
fn execute_run(
    path: &Path,
    override_ping: Option<(&str, &str, u16)>,
) -> Result<PingResult, CliError> {
    let scenario = parse_scenario(path)?;
    let topology = build_topology(&scenario)?;
    println!(
        "NetLab {VERSION}\n\nLoading scenario: {}",
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&scenario.name)
    );
    println!("\nDevices:");
    for host in topology.hosts() {
        println!("  {} {}", host.name(), host.ipv4())
    }
    for switch in topology.switches() {
        println!(
            "  {} (switch, {} ports)",
            switch.name(),
            switch.ports().len()
        )
    }
    println!("\nLinks:");
    for link in topology.links() {
        let (a, b) = link.endpoints();
        let left = topology
            .device_name(
                topology
                    .hosts()
                    .find(|h| h.interface().id() == a)
                    .map(|h| h.id())
                    .or_else(|| {
                        topology
                            .switches()
                            .find(|s| s.ports().iter().any(|p| p.id() == a))
                            .map(|s| s.id())
                    })
                    .unwrap_or(DeviceId(0)),
            )
            .unwrap_or("?");
        let right = topology
            .device_name(
                topology
                    .hosts()
                    .find(|h| h.interface().id() == b)
                    .map(|h| h.id())
                    .or_else(|| {
                        topology
                            .switches()
                            .find(|s| s.ports().iter().any(|p| p.id() == b))
                            .map(|s| s.id())
                    })
                    .unwrap_or(DeviceId(0)),
            )
            .unwrap_or("?");
        println!("  {left} <-> {right}\n  latency: {}ms", link.latency_ms())
    }
    let (from_name, target_text, count) = override_ping.unwrap_or((
        &scenario.action.from,
        &scenario.action.target,
        scenario.action.count,
    ));
    if scenario.version != "0.1" && scenario.version != "0.2" {
        return Err(CliError(format!(
            "unsupported scenario version {}",
            scenario.version
        )));
    }
    let source = topology
        .device_id_by_name(from_name)
        .ok_or_else(|| CliError(format!("unknown source host {from_name}")))?;
    let target: Ipv4Address = target_text
        .parse()
        .map_err(|e| CliError(format!("invalid target IPv4 ({e})")))?;
    println!("\nSimulation initialized.\n\nRunning ping...\n");
    let result = run_ping(
        &topology,
        source,
        target,
        PingConfig {
            count,
            identifier: 1,
        },
    )?;
    print_events(&result, &topology);
    if result.success() {
        println!("\nPing successful.")
    } else {
        println!("\nPing failed.")
    }
    println!(
        "\nPackets:\n  Sent: {}\n  Received: {}\n  Lost: {}",
        result.sent, result.received, result.lost
    );
    if !result.rtts.is_empty() {
        println!("\nRTT:");
        for rtt in &result.rtts {
            println!("  {rtt}ms")
        }
    }
    Ok(result)
}
fn run_cli(args: &[String]) -> Result<(), CliError> {
    match parse_command(args).map_err(CliError) {
        Ok(Command::Help) => {
            print_help();
            Ok(())
        }
        Ok(Command::Version) => {
            println!("NetLab {VERSION}");
            Ok(())
        }
        Ok(Command::Run(path)) => {
            execute_run(Path::new(&path), None)?;
            Ok(())
        }
        Ok(Command::Ping {
            scenario,
            host,
            target,
            count,
        }) => {
            execute_run(Path::new(&scenario), Some((&host, &target, count)))?;
            Ok(())
        }
        Ok(Command::Inspect { scenario, id }) => {
            let result = execute_run(Path::new(&scenario), None)?;
            let selected = packet(&result, id)
                .ok_or_else(|| CliError(format!("packet #{id} was not found")))?;
            println!("\nPacket inspection:\n{selected:#?}");
            Ok(())
        }
        Err(e) => Err(e),
    }
}
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Err(error) = run_cli(&args) {
        eprintln!("error: {error}");
        eprintln!("Try 'netlab --help' for usage information.");
        std::process::exit(2)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_commands() {
        assert_eq!(parse_command(&[]).expect("help"), Command::Help);
        assert_eq!(
            parse_command(&["version".into()]).expect("version"),
            Command::Version
        );
        assert!(parse_command(&["run".into()]).is_err())
    }
}
