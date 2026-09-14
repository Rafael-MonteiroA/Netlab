//! Payloads tipados.
use crate::IcmpMessage;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Payload {
    Icmp(IcmpMessage),
    Raw(Vec<u8>),
}
