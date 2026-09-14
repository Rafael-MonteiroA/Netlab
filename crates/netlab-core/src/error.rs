//! Erros explícitos do motor de simulação.

use crate::time::SimulationTime;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetlabError {
    TimeWentBackwards {
        current: SimulationTime,
        requested: SimulationTime,
    },
    ArithmeticOverflow,
}

impl fmt::Display for NetlabError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimeWentBackwards { current, requested } => write!(
                formatter,
                "event timestamp {requested} is earlier than current time {current}"
            ),
            Self::ArithmeticOverflow => write!(formatter, "simulation arithmetic overflowed"),
        }
    }
}

impl std::error::Error for NetlabError {}
