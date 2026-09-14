//! Link bidirecional entre duas interfaces.
use crate::InterfaceId;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LinkId(pub u64);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    id: LinkId,
    a: InterfaceId,
    b: InterfaceId,
    latency_ms: u64,
    enabled: bool,
}
impl Link {
    pub fn new(id: LinkId, a: InterfaceId, b: InterfaceId, latency_ms: u64, enabled: bool) -> Self {
        Self {
            id,
            a,
            b,
            latency_ms,
            enabled,
        }
    }
    pub fn id(&self) -> LinkId {
        self.id
    }
    pub fn endpoints(&self) -> (InterfaceId, InterfaceId) {
        (self.a, self.b)
    }
    pub fn latency_ms(&self) -> u64 {
        self.latency_ms
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v
    }
    pub fn other(&self, i: InterfaceId) -> Option<InterfaceId> {
        if i == self.a {
            Some(self.b)
        } else if i == self.b {
            Some(self.a)
        } else {
            None
        }
    }
}
