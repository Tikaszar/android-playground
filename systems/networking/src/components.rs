//! ECS Components for networking system internal state

use playground_core_ecs::Component;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use bytes::Bytes;
use async_trait::async_trait;

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

#[async_trait]
impl Component for ConnectionComponent {
    async fn serialize(&self) -> playground_core_ecs::Result<Bytes> {
        bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .map_err(|e| playground_core_ecs::EcsError::SerializationError(e.to_string()))
    }
    
    async fn deserialize(bytes: &Bytes) -> playground_core_ecs::Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| playground_core_ecs::EcsError::SerializationError(e.to_string()))
    }
}

/// Component tracking channel registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelComponent {
    pub channel_id: u16,
    pub channel_name: String,
    pub is_system: bool, // true for channels 1-999, false for 1000+
}

#[async_trait]
impl Component for ChannelComponent {
    async fn serialize(&self) -> playground_core_ecs::Result<Bytes> {
        bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .map_err(|e| playground_core_ecs::EcsError::SerializationError(e.to_string()))
    }
    
    async fn deserialize(bytes: &Bytes) -> playground_core_ecs::Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| playground_core_ecs::EcsError::SerializationError(e.to_string()))
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

#[async_trait]
impl Component for PacketQueueComponent {
    async fn serialize(&self) -> playground_core_ecs::Result<Bytes> {
        // For now, we don't serialize the queues (they're transient)
        Ok(Bytes::new())
    }
    
    async fn deserialize(_bytes: &Bytes) -> playground_core_ecs::Result<Self> {
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

#[async_trait]
impl Component for NetworkStatsComponent {
    async fn serialize(&self) -> playground_core_ecs::Result<Bytes> {
        bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .map_err(|e| playground_core_ecs::EcsError::SerializationError(e.to_string()))
    }
    
    async fn deserialize(bytes: &Bytes) -> playground_core_ecs::Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| playground_core_ecs::EcsError::SerializationError(e.to_string()))
    }
}