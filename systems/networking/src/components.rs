//! ECS Components for networking system internal state

use playground_core_ecs::{ComponentData, EcsError, EcsResult};
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
impl ComponentData for ConnectionComponent {
    async fn serialize(&self) -> EcsResult<Bytes> {
        Ok(bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))?)
    }
    
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
}

/// Component tracking channel registration and packet routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelComponent {
    pub channel_id: u16,
    pub channel_name: String,
    pub handler_active: bool,
}

#[async_trait]
impl ComponentData for ChannelComponent {
    async fn serialize(&self) -> EcsResult<Bytes> {
        Ok(bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))?)
    }
    
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
}

/// Component tracking packet queue for batching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketQueueComponent {
    pub channel_id: u16,
    pub queued_packets: VecDeque<Vec<u8>>,
    pub total_bytes: usize,
    pub priority: u8,
}

impl PacketQueueComponent {
    pub fn new(channel_id: u16, priority: u8) -> Self {
        Self {
            channel_id,
            queued_packets: VecDeque::new(),
            total_bytes: 0,
            priority,
        }
    }
}

#[async_trait]
impl ComponentData for PacketQueueComponent {
    async fn serialize(&self) -> EcsResult<Bytes> {
        Ok(bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))?)
    }
    
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
}

/// Component tracking networking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatsComponent {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors: u64,
    pub last_reset: u64,
}

impl NetworkStatsComponent {
    pub fn new() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            errors: 0,
            last_reset: 0,
        }
    }
}

#[async_trait]
impl ComponentData for NetworkStatsComponent {
    async fn serialize(&self) -> EcsResult<Bytes> {
        Ok(bincode::serialize(self)
            .map(|v| Bytes::from(v))
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))?)
    }
    
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
}