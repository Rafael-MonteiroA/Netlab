//! Mensagens ICMP estruturadas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcmpType {
    EchoRequest,
    EchoReply,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcmpMessage {
    pub icmp_type: IcmpType,
    pub identifier: u16,
    pub sequence_number: u16,
    pub data: Vec<u8>,
}
impl IcmpMessage {
    pub fn echo_request(identifier: u16, sequence: u16) -> Self {
        Self {
            icmp_type: IcmpType::EchoRequest,
            identifier,
            sequence_number: sequence,
            data: Vec::new(),
        }
    }
    pub fn echo_reply(identifier: u16, sequence: u16) -> Self {
        Self {
            icmp_type: IcmpType::EchoReply,
            identifier,
            sequence_number: sequence,
            data: Vec::new(),
        }
    }
    pub fn is_echo_request(&self) -> bool {
        self.icmp_type == IcmpType::EchoRequest
    }
    pub fn is_echo_reply(&self) -> bool {
        self.icmp_type == IcmpType::EchoReply
    }
}
