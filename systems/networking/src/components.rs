//! ECS Components for networking system internal state

use playground_core_ecs::{ComponentData, EcsError};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use bytes::Bytes;

/// Component tracking connection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionComponent {
    pub peer_id: String,
    pub connected: bool,
    pub latency_ms: u32,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub last_activity: u64,
}

impl ComponentData for ConnectionComponent {
    fn serialize(&self) -> Bytes {
        bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .unwrap_or_else(|_| Bytes::new())
    }
    
    fn deserialize(bytes: &Bytes) -> Result<Self, EcsError> {
        bincode::deserialize(bytes)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
}

/// Component tracking channel registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelComponent {
    pub channel_id: u16,
    pub channel_name: String,
    pub is_system: bool, // true for channels 1-999, false for 1000+
}

impl ComponentData for ChannelComponent {
    fn serialize(&self) -> Bytes {
        bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .unwrap_or_else(|_| Bytes::new())
    }
    
    fn deserialize(bytes: &Bytes) -> Result<Self, EcsError> {
        bincode::deserialize(bytes)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
}

/// Component for packet queuing per connection
#[derive(Debug, Clone)]
pub struct PacketQueueComponent {
    pub outgoing: VecDeque<QueuedPacket>,
    pub incoming: VecDeque<QueuedPacket>,
    pub max_queue_size: usize,
}

impl PacketQueueComponent {
    pub fn new() -> Self {
        Self {
            outgoing: VecDeque::new(),
            incoming: VecDeque::new(),
            max_queue_size: 1000,
        }
    }
}

impl ComponentData for PacketQueueComponent {
    fn serialize(&self) -> Bytes {
        // For now, we don't serialize the queues (they're transient)
        Bytes::new()
    }
    
    fn deserialize(_bytes: &Bytes) -> Result<Self, EcsError> {
        Ok(Self::new())
    }
}

/// Queued packet data
#[derive(Debug, Clone)]
pub struct QueuedPacket {
    pub channel_id: u16,
    pub packet_type: u16,
    pub priority: u8,
    pub data: Bytes,
    pub timestamp: u64,
}

/// Component for network statistics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatsComponent {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub average_latency_ms: u32,
    pub last_update: u64,
}

impl ComponentData for NetworkStatsComponent {
    fn serialize(&self) -> Bytes {
        bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .unwrap_or_else(|_| Bytes::new())
    }
    
    fn deserialize(bytes: &Bytes) -> Result<Self, EcsError> {
        bincode::deserialize(bytes)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
}