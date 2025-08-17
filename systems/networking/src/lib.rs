//! Networking System
//! 
//! This system provides networking functionality for Plugins and Apps.
//! Internally uses core/ecs for state management and integrates with
//! core/server for WebSocket communication.

mod components;
mod connection;
mod packet_queue;
mod channel_manager;
mod network_system;

pub use components::*;
pub use connection::*;
pub use packet_queue::*;
pub use channel_manager::*;
pub use network_system::*;

use playground_ecs::{World, EntityId, ComponentBox};
use playground_types::{ChannelId, Priority};
use thiserror::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Invalid network message: {0}")]
    InvalidMessage(String),
    #[error("Network timeout: {0}")]
    Timeout(String),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("ECS error: {0}")]
    EcsError(String),
    #[error("Not connected")]
    NotConnected,
    #[error("Channel not found: {0}")]
    ChannelNotFound(u16),
}

pub type NetworkResult<T> = Result<T, NetworkError>;

/// Main networking system that Plugins interact with
pub struct NetworkingSystem {
    // Internal ECS world for managing network state
    world: Arc<RwLock<World>>,
    // Connection to core/server
    server_handle: Option<Arc<ConnectionManager>>,
    // Channel manager for dynamic registration
    channel_manager: Arc<RwLock<ChannelManager>>,
    // Packet queue for batching
    packet_queue: Arc<RwLock<PacketQueue>>,
}

impl NetworkingSystem {
    /// Create a new networking system
    pub async fn new() -> NetworkResult<Self> {
        let world = Arc::new(RwLock::new(World::new()));
        
        // Register networking components with ECS
        {
            let w = world.write().await;
            w.register_component::<ConnectionComponent>().await;
            w.register_component::<ChannelComponent>().await;
            w.register_component::<PacketQueueComponent>().await;
            w.register_component::<NetworkStatsComponent>().await;
        }
        
        Ok(Self {
            world,
            server_handle: None,
            channel_manager: Arc::new(RwLock::new(ChannelManager::new())),
            packet_queue: Arc::new(RwLock::new(PacketQueue::new())),
        })
    }
    
    /// Initialize and connect to core/server
    pub async fn initialize(&mut self, server_url: Option<String>) -> NetworkResult<()> {
        // Connect to core/server WebSocket endpoint
        let _url = server_url.unwrap_or_else(|| "ws://localhost:3000/ws".to_string());
        
        // TODO: Create actual connection to core/server
        // For now, we'll prepare the internal structure
        
        // Register systems channels (1-999)
        let mut manager = self.channel_manager.write().await;
        manager.register_system_channel("networking", 100).await?;
        
        Ok(())
    }
    
    /// Register a Plugin for a dynamic channel (1000+)
    pub async fn register_plugin(&self, plugin_name: &str) -> NetworkResult<ChannelId> {
        let mut manager = self.channel_manager.write().await;
        manager.register_plugin_channel(plugin_name).await
    }
    
    /// Send a packet to a specific channel with priority
    pub async fn send_packet(
        &self,
        channel: ChannelId,
        packet_type: u16,
        data: Vec<u8>,
        priority: Priority,
    ) -> NetworkResult<()> {
        let mut queue = self.packet_queue.write().await;
        queue.enqueue(channel, packet_type, data, priority).await
    }
    
    /// Process incoming packets for a channel
    pub async fn receive_packets(&self, channel: ChannelId) -> NetworkResult<Vec<IncomingPacket>> {
        let queue = self.packet_queue.read().await;
        queue.get_incoming(channel).await
    }
    
    /// Create a peer-to-peer connection entity
    pub async fn create_connection(&self, peer_id: String) -> NetworkResult<EntityId> {
        let mut world = self.world.write().await;
        
        let components: Vec<ComponentBox> = vec![
            Box::new(ConnectionComponent {
                peer_id: peer_id.clone(),
                connected: false,
                latency_ms: 0,
                packets_sent: 0,
                packets_received: 0,
            }),
            Box::new(PacketQueueComponent::new()),
        ];
        
        let entities = world.spawn_batch(vec![components])
            .await
            .map_err(|e| NetworkError::EcsError(e.to_string()))?;
        
        entities.into_iter()
            .next()
            .ok_or_else(|| NetworkError::EcsError("Failed to create entity".to_string()))
    }
    
    /// Update connection status
    pub async fn update_connection(&self, entity: EntityId, connected: bool) -> NetworkResult<()> {
        let world = self.world.read().await;
        
        // Get the component and update it
        // Note: core/ecs doesn't have a direct way to update components in place
        // We would need to remove and re-add the component with updated values
        // For now, we'll leave this as a TODO
        
        // TODO: Implement component update when we have the proper API
        let _ = (entity, connected, world);
        
        Ok(())
    }
    
    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkResult<NetworkStats> {
        let _world = self.world.read().await;
        
        // Query all connection entities for stats
        // TODO: Use proper query API when available
        let total_connections = 0;
        let active_connections = 0;
        let total_packets_sent = 0u64;
        let total_packets_received = 0u64;
        
        Ok(NetworkStats {
            total_connections,
            active_connections,
            total_packets_sent,
            total_packets_received,
            channels_registered: self.channel_manager.read().await.count(),
        })
    }
    
    /// Process one frame of network updates (called at 60fps)
    pub async fn update(&self, _delta_time: f32) -> NetworkResult<()> {
        // Process outgoing packet queue
        let mut queue = self.packet_queue.write().await;
        let batched_packets = queue.flush_frame().await?;
        
        // Send batched packets through core/server
        if let Some(_server) = &self.server_handle {
            for (channel, packets) in batched_packets {
                // TODO: Send through actual server connection
                let _ = (channel, packets);
            }
        }
        
        // Update connection stats
        // TODO: Use proper query API when available
        
        Ok(())
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub total_packets_sent: u64,
    pub total_packets_received: u64,
    pub channels_registered: usize,
}

/// Incoming packet data
#[derive(Debug, Clone)]
pub struct IncomingPacket {
    pub channel: ChannelId,
    pub packet_type: u16,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

impl Default for NetworkingSystem {
    fn default() -> Self {
        futures::executor::block_on(async {
            Self::new().await.expect("Failed to create NetworkingSystem")
        })
    }
}