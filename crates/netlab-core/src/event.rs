//! Eventos fortemente tipados do motor.

use crate::time::SimulationTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventType {
    PacketCreated {
        packet_id: u64,
    },
    PacketSent {
        packet_id: u64,
        from_device: u64,
        link_id: u64,
    },
    PacketTransmitting {
        packet_id: u64,
        link_id: u64,
        from_device: u64,
    },
    PacketReceived {
        packet_id: u64,
        at_device: u64,
        ingress_link: u64,
    },
    PacketDropped {
        packet_id: u64,
        reason: String,
    },
    DeviceOnline {
        device_id: u64,
    },
    DeviceOffline {
        device_id: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulationEvent {
    pub id: EventId,
    pub timestamp: SimulationTime,
    pub event_type: EventType,
}

impl SimulationEvent {
    pub(crate) fn new(id: EventId, timestamp: SimulationTime, event_type: EventType) -> Self {
        Self {
            id,
            timestamp,
            event_type,
        }
    }
}

pub type Event = SimulationEvent;
