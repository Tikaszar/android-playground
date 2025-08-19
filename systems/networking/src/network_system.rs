//! Network system implementation using core/ecs

use crate::{NetworkError, NetworkResult};
use playground_core_ecs::World;
use playground_core_types::{ChannelId, Priority, Packet};
use std::sync::Arc;
use tokio::sync::RwLock;
use bytes::{BytesMut, BufMut};

/// Internal network system that processes ECS queries
pub struct NetworkSystem {
    world: Arc<RwLock<World>>,
}

impl NetworkSystem {
    pub fn new(world: Arc<RwLock<World>>) -> Self {
        Self { world }
    }
    
    /// Process network events using ECS queries
    pub async fn process_network_events(&self) -> NetworkResult<()> {
        let world = self.world.read().await;
        
        // Query all connection entities
        // Note: core/ecs uses a different query API than expected
        // We'll need to adapt to the actual API
        
        // for conn in connections {
        //     if conn.connected {
        //         // Process active connections
        //         tracing::trace!("Processing connection: {}", conn.peer_id);
        //     }
        // }
        
        Ok(())
    }
    
    /// Create a packet from components
    pub async fn create_packet(
        &self,
        channel: ChannelId,
        packet_type: u16,
        data: Vec<u8>,
        priority: Priority,
    ) -> NetworkResult<Packet> {
        let mut bytes = BytesMut::new();
        
        // Write packet header
        bytes.extend_from_slice(&channel.to_le_bytes());
        bytes.extend_from_slice(&packet_type.to_le_bytes());
        bytes.put_u8(priority as u8);
        
        // Write payload size and data
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&data);
        
        Ok(Packet {
            channel_id: channel,
            packet_type,
            priority: priority as u8,
            payload_size: data.len() as u32,
            payload: data,
        })
    }
    
    /// Parse an incoming packet
    pub async fn parse_packet(&self, data: &[u8]) -> NetworkResult<Packet> {
        if data.len() < 9 {
            return Err(NetworkError::InvalidMessage(
                "Packet too small".to_string()
            ));
        }
        
        let channel_id = u16::from_le_bytes([data[0], data[1]]);
        let packet_type = u16::from_le_bytes([data[2], data[3]]);
        let priority = data[4];
        let payload_size = u32::from_le_bytes([data[5], data[6], data[7], data[8]]);
        
        if data.len() < 9 + payload_size as usize {
            return Err(NetworkError::InvalidMessage(
                "Payload size mismatch".to_string()
            ));
        }
        
        let payload = data[9..9 + payload_size as usize].to_vec();
        
        Ok(Packet {
            channel_id,
            packet_type,
            priority,
            payload_size,
            payload,
        })
    }
    
    /// Update network statistics components
    pub async fn update_stats(&self) -> NetworkResult<()> {
        let mut world = self.world.write().await;
        
        // Query all network stats components
        // Note: core/ecs uses a different query API than expected
        // We'll need to adapt to the actual API
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // TODO: Update stats when we have the proper query API
        
        Ok(())
    }
}