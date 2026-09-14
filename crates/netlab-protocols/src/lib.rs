//! Comportamento dos protocolos sobre pacotes e topologia.
pub mod icmp;
pub use icmp::{run_ping, IcmpHandler, PingConfig, PingResult, ProtocolError};
