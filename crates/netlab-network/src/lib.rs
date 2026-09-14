//! Modelo de topologia virtual. Este crate não executa eventos.
use std::fmt;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkError {
    InvalidAddress(String),
    DuplicateDeviceId,
    DeviceNotFound(u64),
    LinkNotFound(u64),
    DeviceAlreadyConnected,
    NoAvailableInterface,
    InvalidLatency,
}
impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress(v) => write!(f, "invalid network address: {v}"),
            Self::DuplicateDeviceId => write!(f, "device id already exists"),
            Self::DeviceNotFound(id) => write!(f, "device {id} was not found"),
            Self::LinkNotFound(id) => write!(f, "link {id} was not found"),
            Self::DeviceAlreadyConnected => write!(f, "device is already connected"),
            Self::NoAvailableInterface => write!(f, "device has no available interface"),
            Self::InvalidLatency => write!(f, "latency must be finite and non-negative"),
        }
    }
}
impl std::error::Error for NetworkError {}
pub mod device;
pub mod interface;
pub mod ipv4;
pub mod link;
pub mod mac;
pub mod network;
pub mod topology;
pub use device::{DeviceId, DeviceState, Host, Switch};
pub use interface::{Interface, InterfaceId};
pub use ipv4::Ipv4Address;
pub use link::{Link, LinkId};
pub use mac::MacAddress;
pub use network::Network;
pub use topology::Topology;
