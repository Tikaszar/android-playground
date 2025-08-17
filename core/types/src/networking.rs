//! Networking types shared across the system

use serde::{Serialize, Deserialize};

/// Channel ID type
pub type ChannelId = u16;

/// Priority levels for packet queuing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
    Blocker = 4,
}

impl TryFrom<u8> for Priority {
    type Error = String;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Priority::Low),
            1 => Ok(Priority::Medium),
            2 => Ok(Priority::High),
            3 => Ok(Priority::Critical),
            4 => Ok(Priority::Blocker),
            _ => Err(format!("Invalid priority value: {}", value)),
        }
    }
}

/// Network packet structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub channel_id: ChannelId,
    pub packet_type: u16,
    pub priority: u8,
    pub payload_size: u32,
    pub payload: Vec<u8>,
}

/// Control message types for channel 0
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ControlMessageType {
    RegisterSystem = 1,
    RegisterPlugin = 2,
    QueryChannel = 3,
    ListChannels = 4,
    RegisterResponse = 5,
    QueryResponse = 6,
    ListResponse = 7,
    Error = 255,
}