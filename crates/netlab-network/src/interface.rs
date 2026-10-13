//! Interface de rede de um dispositivo.
use crate::link::LinkId;
use crate::{Ipv4Address, MacAddress};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InterfaceId(pub u64);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interface {
    id: InterfaceId,
    name: String,
    mac: MacAddress,
    ipv4: Ipv4Address,
    link_id: Option<LinkId>,
}
impl Interface {
    pub fn new(
        id: InterfaceId,
        name: impl Into<String>,
        mac: MacAddress,
        ipv4: Ipv4Address,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            mac,
            ipv4,
            link_id: None,
        }
    }
    pub fn id(&self) -> InterfaceId {
        self.id
    }
    pub(crate) fn set_id(&mut self, id: InterfaceId) {
        self.id = id;
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn mac(&self) -> MacAddress {
        self.mac
    }
    pub fn ipv4(&self) -> Ipv4Address {
        self.ipv4
    }
    pub fn link_id(&self) -> Option<LinkId> {
        self.link_id
    }
    pub fn is_connected(&self) -> bool {
        self.link_id.is_some()
    }
    pub(crate) fn attach(&mut self, id: LinkId) {
        self.link_id = Some(id)
    }
}
