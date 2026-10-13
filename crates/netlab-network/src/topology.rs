//! Agregador determinístico de hosts, switches e links.

use crate::interface::InterfaceId;
use crate::{
    DeviceId, DeviceState, Host, Ipv4Address, Link, LinkId, MacAddress, NetworkError, Switch,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Topology {
    hosts: BTreeMap<DeviceId, Host>,
    switches: BTreeMap<DeviceId, Switch>,
    links: BTreeMap<LinkId, Link>,
    next_device_id: u64,
    next_interface_id: u64,
    next_link_id: u64,
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}

impl Topology {
    pub fn new() -> Self {
        Self {
            hosts: BTreeMap::new(),
            switches: BTreeMap::new(),
            links: BTreeMap::new(),
            next_device_id: 1,
            next_interface_id: 1,
            next_link_id: 1,
        }
    }

    pub fn insert_host(&mut self, mut host: Host) -> Result<DeviceId, NetworkError> {
        let id = host.id();
        if self.hosts.contains_key(&id) || self.switches.contains_key(&id) {
            return Err(NetworkError::DuplicateDeviceId);
        }
        self.next_device_id = self.next_device_id.max(id.0.saturating_add(1));
        // Host::new uses its device ID as a provisional interface ID. Switch ports
        // may already own that ID, so allocate a distinct ID before insertion.
        let interface_id = host.interface().id();
        if self
            .hosts
            .values()
            .flat_map(Host::interfaces)
            .chain(self.switches.values().flat_map(Switch::ports))
            .any(|interface| interface.id() == interface_id)
        {
            host.interface_mut()
                .set_id(InterfaceId(self.next_interface_id));
        }
        self.next_interface_id = self
            .next_interface_id
            .max(host.interface().id().0.saturating_add(1));
        self.hosts.insert(id, host);
        Ok(id)
    }

    pub fn add_host(
        &mut self,
        name: impl Into<String>,
        mac: MacAddress,
        ip: Ipv4Address,
    ) -> Result<DeviceId, NetworkError> {
        let id = DeviceId(self.next_device_id);
        self.next_device_id = self.next_device_id.saturating_add(1);
        self.insert_host(Host::new(id, name, mac, ip))
    }

    pub fn add_switch(
        &mut self,
        name: impl Into<String>,
        port_count: usize,
    ) -> Result<DeviceId, NetworkError> {
        if port_count == 0 {
            return Err(NetworkError::NoAvailableInterface);
        }
        let id = DeviceId(self.next_device_id);
        self.next_device_id = self.next_device_id.saturating_add(1);
        let port_ids = (0..port_count)
            .map(|_| {
                let port_id = crate::InterfaceId(self.next_interface_id);
                self.next_interface_id = self.next_interface_id.saturating_add(1);
                port_id
            })
            .collect::<Vec<_>>();
        self.switches
            .insert(id, Switch::with_port_ids(id, name, port_ids));
        Ok(id)
    }

    pub fn host(&self, id: DeviceId) -> Option<&Host> {
        self.hosts.get(&id)
    }
    pub fn host_mut(&mut self, id: DeviceId) -> Option<&mut Host> {
        self.hosts.get_mut(&id)
    }
    pub fn hosts(&self) -> impl Iterator<Item = &Host> {
        self.hosts.values()
    }
    pub fn switch(&self, id: DeviceId) -> Option<&Switch> {
        self.switches.get(&id)
    }
    pub fn switch_mut(&mut self, id: DeviceId) -> Option<&mut Switch> {
        self.switches.get_mut(&id)
    }
    pub fn switches(&self) -> impl Iterator<Item = &Switch> {
        self.switches.values()
    }
    pub fn is_switch(&self, id: DeviceId) -> bool {
        self.switches.contains_key(&id)
    }
    pub fn device_name(&self, id: DeviceId) -> Option<&str> {
        self.host(id)
            .map(Host::name)
            .or_else(|| self.switch(id).map(Switch::name))
    }
    pub fn device_is_online(&self, id: DeviceId) -> bool {
        self.host(id).is_some_and(Host::is_online) || self.switch(id).is_some_and(Switch::is_online)
    }
    pub fn host_by_name(&self, name: &str) -> Option<&Host> {
        self.hosts.values().find(|host| host.name() == name)
    }
    pub fn host_by_ip(&self, ip: Ipv4Address) -> Option<&Host> {
        self.hosts.values().find(|host| host.ipv4() == ip)
    }
    pub fn device_id_by_name(&self, name: &str) -> Option<DeviceId> {
        self.host_by_name(name).map(Host::id).or_else(|| {
            self.switches
                .values()
                .find(|switch| switch.name() == name)
                .map(Switch::id)
        })
    }

    fn interface_ids(&self, device: DeviceId) -> Option<Vec<InterfaceId>> {
        if let Some(host) = self.host(device) {
            return Some(vec![host.interface().id()]);
        }
        self.switch(device)
            .map(|switch| switch.ports().iter().map(|port| port.id()).collect())
    }

    fn interface_is_connected(&self, interface: InterfaceId) -> bool {
        self.hosts
            .values()
            .flat_map(Host::interfaces)
            .chain(self.switches.values().flat_map(Switch::ports))
            .find(|candidate| candidate.id() == interface)
            .is_some_and(|candidate| candidate.is_connected())
    }

    fn available_interface(&self, device: DeviceId) -> Option<InterfaceId> {
        self.interface_ids(device)?
            .into_iter()
            .find(|interface| !self.interface_is_connected(*interface))
    }

    fn attach_interface(
        &mut self,
        device: DeviceId,
        interface: InterfaceId,
        link: LinkId,
    ) -> Result<(), NetworkError> {
        if let Some(host) = self.host_mut(device) {
            host.interface_mut().attach(link);
            return Ok(());
        }
        if let Some(switch) = self.switch_mut(device) {
            switch
                .port_mut(interface)
                .ok_or(NetworkError::NoAvailableInterface)?
                .attach(link);
            return Ok(());
        }
        Err(NetworkError::DeviceNotFound(device.0))
    }

    pub fn connect(
        &mut self,
        a: DeviceId,
        b: DeviceId,
        latency: u64,
        enabled: bool,
    ) -> Result<LinkId, NetworkError> {
        if a == b {
            return Err(NetworkError::DeviceAlreadyConnected);
        }
        let ia = self.available_interface(a).ok_or_else(|| {
            if self.interface_ids(a).is_some() {
                NetworkError::NoAvailableInterface
            } else {
                NetworkError::DeviceNotFound(a.0)
            }
        })?;
        let ib = self.available_interface(b).ok_or_else(|| {
            if self.interface_ids(b).is_some() {
                NetworkError::NoAvailableInterface
            } else {
                NetworkError::DeviceNotFound(b.0)
            }
        })?;
        let id = LinkId(self.next_link_id);
        self.next_link_id = self.next_link_id.saturating_add(1);
        self.links
            .insert(id, Link::new(id, ia, ib, latency, enabled));
        self.attach_interface(a, ia, id)?;
        self.attach_interface(b, ib, id)?;
        Ok(id)
    }

    pub fn connect_hosts(
        &mut self,
        a: DeviceId,
        b: DeviceId,
        latency: u64,
    ) -> Result<LinkId, NetworkError> {
        self.connect(a, b, latency, true)
    }

    pub fn link(&self, id: LinkId) -> Option<&Link> {
        self.links.get(&id)
    }
    pub fn link_mut(&mut self, id: LinkId) -> Option<&mut Link> {
        self.links.get_mut(&id)
    }
    pub fn links(&self) -> impl Iterator<Item = &Link> {
        self.links.values()
    }
    pub fn link_for_interface(&self, interface: InterfaceId) -> Option<&Link> {
        self.links
            .values()
            .find(|link| link.endpoints().0 == interface || link.endpoints().1 == interface)
    }
    pub fn interface_for_device_on_link(
        &self,
        device: DeviceId,
        link: LinkId,
    ) -> Option<InterfaceId> {
        let link = self.link(link)?;
        self.interface_ids(device)?
            .into_iter()
            .find(|interface| link.endpoints().0 == *interface || link.endpoints().1 == *interface)
    }
    pub fn connected_links(&self, device: DeviceId) -> Vec<&Link> {
        let Some(interfaces) = self.interface_ids(device) else {
            return Vec::new();
        };
        self.links
            .values()
            .filter(|link| {
                let (a, b) = link.endpoints();
                interfaces.contains(&a) || interfaces.contains(&b)
            })
            .collect()
    }
    pub fn link_between(&self, a: DeviceId, b: DeviceId) -> Option<&Link> {
        self.connected_links(a)
            .into_iter()
            .find(|link| self.peer(link.id(), a) == Some(b))
    }
    pub fn peer(&self, id: LinkId, device: DeviceId) -> Option<DeviceId> {
        let link = self.link(id)?;
        let interface = self.interface_for_device_on_link(device, id)?;
        let other = link.other(interface)?;
        self.hosts
            .values()
            .find(|host| host.interface().id() == other)
            .map(Host::id)
            .or_else(|| {
                self.switches
                    .values()
                    .find(|switch| switch.ports().iter().any(|port| port.id() == other))
                    .map(Switch::id)
            })
    }
    pub fn set_device_state(
        &mut self,
        id: DeviceId,
        state: DeviceState,
    ) -> Result<(), NetworkError> {
        if let Some(host) = self.host_mut(id) {
            host.set_state(state);
            return Ok(());
        }
        if let Some(switch) = self.switch_mut(id) {
            switch.set_state(state);
            return Ok(());
        }
        Err(NetworkError::DeviceNotFound(id.0))
    }
}

#[cfg(test)]
mod tests {
    use super::Topology;
    use crate::{Ipv4Address, MacAddress};

    fn host(topology: &mut Topology, name: &str, last: u8) -> crate::DeviceId {
        topology
            .add_host(
                name,
                MacAddress::new([2, 0, 0, 0, 0, last]),
                Ipv4Address::new([10, 0, 0, last]),
            )
            .expect("host")
    }

    #[test]
    fn interfaces_remain_unique_when_hosts_are_added_after_switches() {
        let mut topology = Topology::new();
        let switch = topology.add_switch("SW1", 4).expect("switch");
        let a = host(&mut topology, "PC1", 1);
        let b = host(&mut topology, "PC2", 2);
        let link_a = topology.connect(a, switch, 5, true).expect("link A");
        let link_b = topology.connect(b, switch, 7, true).expect("link B");
        assert_eq!(topology.peer(link_a, a), Some(switch));
        assert_eq!(topology.peer(link_b, switch), Some(b));
        let interfaces: Vec<_> = topology
            .hosts()
            .flat_map(|h| h.interfaces())
            .chain(topology.switches().flat_map(|s| s.ports()))
            .map(|i| i.id())
            .collect();
        let unique: std::collections::HashSet<_> = interfaces.iter().collect();
        assert_eq!(interfaces.len(), unique.len());
    }

    #[test]
    fn connects_hosts() {
        let mut topology = Topology::new();
        let a = host(&mut topology, "PC1", 1);
        let b = host(&mut topology, "PC2", 2);
        assert_eq!(topology.connect_hosts(a, b, 10).expect("link").0, 1);
    }

    #[test]
    fn switch_allocates_ports_and_connects_multiple_hosts() {
        let mut topology = Topology::new();
        let a = host(&mut topology, "PC1", 1);
        let b = host(&mut topology, "PC2", 2);
        let switch = topology.add_switch("SW1", 2).expect("switch");
        topology.connect(a, switch, 5, true).expect("first link");
        topology.connect(b, switch, 7, true).expect("second link");
        assert_eq!(topology.connected_links(switch).len(), 2);
        let first_link = topology.connected_links(switch)[0].id();
        assert_eq!(topology.peer(first_link, switch), Some(a));
    }

    #[test]
    fn switch_learns_source_and_returns_known_destination_port() {
        let mut topology = Topology::new();
        let a = host(&mut topology, "PC1", 1);
        let b = host(&mut topology, "PC2", 2);
        let switch = topology.add_switch("SW1", 2).expect("switch");
        let link_a = topology.connect(a, switch, 5, true).expect("first link");
        let link_b = topology.connect(b, switch, 5, true).expect("second link");
        let ingress_a = topology
            .interface_for_device_on_link(switch, link_a)
            .expect("switch ingress");
        let ingress_b = topology
            .interface_for_device_on_link(switch, link_b)
            .expect("switch egress");
        let destination = MacAddress::new([2, 0, 0, 0, 0, 2]);
        let source = MacAddress::new([2, 0, 0, 0, 0, 1]);
        let flooded = topology.switch_mut(switch).expect("switch").forward_ports(
            source,
            destination,
            ingress_a,
        );
        assert_eq!(flooded, vec![ingress_b]);
        let returned = topology.switch_mut(switch).expect("switch").forward_ports(
            destination,
            source,
            ingress_b,
        );
        assert_eq!(returned, vec![ingress_a]);
    }
}
