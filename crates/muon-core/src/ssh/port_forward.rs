#[derive(Debug, Clone, PartialEq)]
pub enum ForwardDirection {
    Local,
    Remote,
}

pub struct PortForwardRule {
    pub bind_host: String,
    pub bind_port: u16,
    pub target_host: String,
    pub target_port: u16,
    pub direction: ForwardDirection,
}
