//! Ciclo de execução do engine, sem dependência de rede ou apresentação.

use crate::config::SimulationConfig;
use crate::error::NetlabError;
use crate::event::{EventId, EventType, SimulationEvent};
use crate::scheduler::Scheduler;
use crate::time::SimulationTime;

#[derive(Debug)]
pub struct Simulation {
    pub config: SimulationConfig,
    scheduler: Scheduler,
    processed_events: Vec<SimulationEvent>,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            config,
            scheduler: Scheduler::new(),
            processed_events: Vec::new(),
        }
    }

    pub fn now(&self) -> SimulationTime {
        self.scheduler.now()
    }
    pub fn pending_events(&self) -> usize {
        self.scheduler.pending()
    }
    pub fn events(&self) -> &[SimulationEvent] {
        &self.processed_events
    }

    pub fn schedule_at(
        &mut self,
        timestamp: SimulationTime,
        event_type: EventType,
    ) -> Result<EventId, NetlabError> {
        self.scheduler.schedule_at(timestamp, event_type)
    }

    pub fn schedule_after(
        &mut self,
        delay_ms: u64,
        event_type: EventType,
    ) -> Result<EventId, NetlabError> {
        self.scheduler.schedule_after(delay_ms, event_type)
    }

    pub fn step(&mut self) -> Option<SimulationEvent> {
        let event = self.scheduler.step()?;
        self.processed_events.push(event.clone());
        Some(event)
    }

    pub fn run(&mut self) -> Vec<SimulationEvent> {
        while self.step().is_some() {}
        self.processed_events.clone()
    }
}
