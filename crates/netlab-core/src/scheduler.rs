//! Fila determinística e scheduler single-threaded.

use crate::error::NetlabError;
use crate::event::{EventId, EventType, SimulationEvent};
use crate::time::SimulationTime;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Debug)]
struct QueuedEvent(SimulationEvent);

impl PartialEq for QueuedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}
impl Eq for QueuedEvent {}
impl PartialOrd for QueuedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for QueuedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .0
            .timestamp
            .cmp(&self.0.timestamp)
            .then_with(|| other.0.id.cmp(&self.0.id))
    }
}

#[derive(Debug, Default)]
pub struct EventQueue {
    events: BinaryHeap<QueuedEvent>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, event: SimulationEvent) {
        self.events.push(QueuedEvent(event));
    }
    pub fn pop(&mut self) -> Option<SimulationEvent> {
        self.events.pop().map(|event| event.0)
    }
    pub fn peek(&self) -> Option<&SimulationEvent> {
        self.events.peek().map(|event| &event.0)
    }
    pub fn len(&self) -> usize {
        self.events.len()
    }
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[derive(Debug)]
pub struct Scheduler {
    now: SimulationTime,
    next_event_id: u64,
    queue: EventQueue,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            now: SimulationTime::ZERO,
            next_event_id: 1,
            queue: EventQueue::new(),
        }
    }

    pub fn now(&self) -> SimulationTime {
        self.now
    }
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    pub fn schedule_at(
        &mut self,
        timestamp: SimulationTime,
        event_type: EventType,
    ) -> Result<EventId, NetlabError> {
        if timestamp < self.now {
            return Err(NetlabError::TimeWentBackwards {
                current: self.now,
                requested: timestamp,
            });
        }
        let id = EventId(self.next_event_id);
        self.next_event_id = self
            .next_event_id
            .checked_add(1)
            .ok_or(NetlabError::ArithmeticOverflow)?;
        self.queue
            .push(SimulationEvent::new(id, timestamp, event_type));
        Ok(id)
    }

    pub fn schedule_after(
        &mut self,
        delay_ms: u64,
        event_type: EventType,
    ) -> Result<EventId, NetlabError> {
        let timestamp = self.now + delay_ms;
        self.schedule_at(timestamp, event_type)
    }

    pub fn step(&mut self) -> Option<SimulationEvent> {
        let event = self.queue.pop()?;
        self.now = event.timestamp;
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::{EventQueue, Scheduler};
    use crate::event::EventType;
    use crate::time::SimulationTime;

    #[test]
    fn orders_by_time_and_then_event_id() {
        let mut queue = EventQueue::new();
        let mut scheduler = Scheduler::new();
        scheduler
            .schedule_at(
                SimulationTime::from_millis(10),
                EventType::DeviceOnline { device_id: 1 },
            )
            .expect("schedule");
        scheduler
            .schedule_at(
                SimulationTime::ZERO,
                EventType::DeviceOffline { device_id: 1 },
            )
            .expect("schedule");
        while let Some(event) = scheduler.step() {
            queue.push(event);
        }
        assert_eq!(queue.pop().expect("event").timestamp, SimulationTime::ZERO);
    }

    #[test]
    fn rejects_events_in_the_past() {
        let mut scheduler = Scheduler::new();
        scheduler
            .schedule_at(
                SimulationTime::from_millis(5),
                EventType::DeviceOnline { device_id: 1 },
            )
            .expect("schedule");
        scheduler.step();
        assert!(scheduler
            .schedule_at(
                SimulationTime::ZERO,
                EventType::DeviceOffline { device_id: 1 }
            )
            .is_err());
    }
}
