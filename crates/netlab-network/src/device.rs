//! Dispositivos e hosts da v0.1.
use crate::interface::{Interface, InterfaceId};
use crate::{Ipv4Address, MacAddress};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceState {
    Online,
    Offline,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Host {
    id: DeviceId,
    name: String,
    interfaces: Vec<Interface>,
    state: DeviceState,
}
impl Host {
    pub fn new(id: DeviceId, name: impl Into<String>, mac: MacAddress, ip: Ipv4Address) -> Self {
        Self {
            id,
            name: name.into(),
            interfaces: vec![Interface::new(InterfaceId(id.0), "eth0", mac, ip)],
            state: DeviceState::Online,
        }
    }
    pub fn id(&self) -> DeviceId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn state(&self) -> DeviceState {
        self.state
    }
    pub fn is_online(&self) -> bool {
        self.state == DeviceState::Online
    }
    pub fn set_state(&mut self, s: DeviceState) {
        self.state = s
    }
    pub fn interfaces(&self) -> &[Interface] {
        &self.interfaces
    }
    pub fn interface(&self) -> &Interface {
        &self.interfaces[0]
    }
    pub fn interface_mut(&mut self) -> &mut Interface {
        &mut self.interfaces[0]
    }
    pub fn mac(&self) -> MacAddress {
        self.interface().mac()
    }
    pub fn ipv4(&self) -> Ipv4Address {
        self.interface().ipv4()
    }
}

/// Switch Ethernet de camada 2.
///
/// O switch não possui um endereço IP próprio. Cada porta é modelada como
/// uma interface para que links possam ser conectados à topologia da mesma
/// forma que em um host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Switch {
    id: DeviceId,
    name: String,
    ports: Vec<Interface>,
    state: DeviceState,
    mac_table: BTreeMap<MacAddress, InterfaceId>,
}

impl Switch {
    pub fn new(id: DeviceId, name: impl Into<String>, port_count: usize) -> Self {
        let base = id.0.saturating_mul(1_000_000);
        Self::with_port_ids(
            id,
            name,
            (0..port_count).map(|index| InterfaceId(base.saturating_add(index as u64 + 1))),
        )
    }

    pub(crate) fn with_port_ids(
        id: DeviceId,
        name: impl Into<String>,
        port_ids: impl IntoIterator<Item = InterfaceId>,
    ) -> Self {
        let ports = port_ids
            .into_iter()
            .enumerate()
            .map(|(index, port_id)| {
                Interface::new(
                    port_id,
                    format!("port{}", index + 1),
                    MacAddress::new([0; 6]),
                    Ipv4Address::new([0; 4]),
                )
            })
            .collect();
        Self {
            id,
            name: name.into(),
            ports,
            state: DeviceState::Online,
            mac_table: BTreeMap::new(),
        }
    }

    pub fn id(&self) -> DeviceId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn state(&self) -> DeviceState {
        self.state
    }
    pub fn is_online(&self) -> bool {
        self.state == DeviceState::Online
    }
    pub fn set_state(&mut self, state: DeviceState) {
        self.state = state;
    }
    pub fn ports(&self) -> &[Interface] {
        &self.ports
    }
    pub fn port(&self, id: InterfaceId) -> Option<&Interface> {
        self.ports.iter().find(|port| port.id() == id)
    }
    pub fn port_mut(&mut self, id: InterfaceId) -> Option<&mut Interface> {
        self.ports.iter_mut().find(|port| port.id() == id)
    }

    /// Aprende a origem na porta de entrada e retorna as portas de saída.
    /// Destinos desconhecidos e broadcast são inundados para todas as portas
    /// conectadas, exceto a porta de entrada.
    pub fn forward_ports(
        &mut self,
        source: MacAddress,
        destination: MacAddress,
        ingress: InterfaceId,
    ) -> Vec<InterfaceId> {
        self.mac_table.insert(source, ingress);
        if destination == MacAddress::broadcast() {
            return self
                .ports
                .iter()
                .filter(|port| port.id() != ingress && port.is_connected())
                .map(Interface::id)
                .collect();
        }
        match self.mac_table.get(&destination).copied() {
            Some(port)
                if port != ingress && self.port(port).is_some_and(Interface::is_connected) =>
            {
                vec![port]
            }
            Some(_) => Vec::new(),
            None => self
                .ports
                .iter()
                .filter(|port| port.id() != ingress && port.is_connected())
                .map(Interface::id)
                .collect(),
        }
    }

    pub fn mac_table(&self) -> &BTreeMap<MacAddress, InterfaceId> {
        &self.mac_table
    }
}
