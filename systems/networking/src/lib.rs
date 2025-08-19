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
mod websocket_client;

pub use components::*;
pub use connection::*;
pub use packet_queue::*;
pub use channel_manager::*;
pub use network_system::*;
use websocket_client::WebSocketClient;

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
    // WebSocket client connection to core/server
    ws_client: Option<Arc<WebSocketClient>>,
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
            ws_client: None,
            channel_manager: Arc::new(RwLock::new(ChannelManager::new())),
            packet_queue: Arc::new(RwLock::new(PacketQueue::new())),
        })
    }
    
    /// Initialize and connect to core/server
    pub async fn initialize(&mut self, server_url: Option<String>) -> NetworkResult<()> {
        // Connect to core/server WebSocket endpoint
        let url = server_url.unwrap_or_else(|| "ws://localhost:8080/ws".to_string());
        
        // Create and connect WebSocket client
        let client = Arc::new(WebSocketClient::new(url));
        client.connect().await?;
        
        // Store the client
        self.ws_client = Some(client.clone());
        
        // Register systems channels (1-999)
        let mut manager = self.channel_manager.write().await;
        let channel_id = manager.register_system_channel("networking", 100).await?;
        
        // Register with the WebSocket server
        if let Some(ws) = &self.ws_client {
            ws.register_channel(channel_id, "networking").await?;
        }
        
        Ok(())
    }
    
    /// Register a Plugin for a dynamic channel (1000+)
    pub async fn register_plugin(&self, plugin_name: &str) -> NetworkResult<ChannelId> {
        let mut manager = self.channel_manager.write().await;
        let channel_id = manager.register_plugin_channel(plugin_name).await?;
        
        // Register with the WebSocket server
        if let Some(ws) = &self.ws_client {
            ws.register_channel(channel_id, plugin_name).await?;
        }
        
        Ok(channel_id)
    }
    
    /// Send a packet to a specific channel with priority
    pub async fn send_packet(
        &self,
        channel: ChannelId,
        packet_type: u16,
        data: Vec<u8>,
        priority: Priority,
    ) -> NetworkResult<()> {
        // Send directly through WebSocket if connected
        if let Some(ws) = &self.ws_client {
            use playground_server::packet::Packet;
            use bytes::Bytes;
            
            // Convert Priority type
            let server_priority = match priority {
                Priority::Low => playground_server::Priority::Low,
                Priority::Medium => playground_server::Priority::Medium,
                Priority::High => playground_server::Priority::High,
                Priority::Critical => playground_server::Priority::Critical,
                Priority::Blocker => playground_server::Priority::Blocker,
            };
            
            let packet = Packet {
                channel_id: channel,
                packet_type,
                priority: server_priority,
                payload: Bytes::from(data.clone()),
            };
            
            ws.send_packet(packet).await?;
        }
        
        // Also queue it locally for tracking
        let mut queue = self.packet_queue.write().await;
        queue.enqueue(channel, packet_type, data, priority).await
    }
    
    /// Process incoming packets for a channel
    pub async fn receive_packets(&self, channel: ChannelId) -> NetworkResult<Vec<IncomingPacket>> {
        let mut result = Vec::new();
        
        // Get packets from WebSocket
        if let Some(ws) = &self.ws_client {
            let packets = ws.receive_packets().await;
            
            // Filter for requested channel and convert to IncomingPacket
            for packet in packets {
                if packet.channel_id == channel {
                    result.push(IncomingPacket {
                        channel,
                        packet_type: packet.packet_type,
                        data: packet.payload.to_vec(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    });
                }
            }
        }
        
        // Also check local queue
        let queue = self.packet_queue.read().await;
        let local_packets = queue.get_incoming(channel).await?;
        result.extend(local_packets);
        
        Ok(result)
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
        // Check WebSocket connection status
        if let Some(ws) = &self.ws_client {
            if !ws.is_connected().await {
                // TODO: Handle reconnection
                return Ok(());
            }
            
            // Process any incoming packets from WebSocket
            let packets = ws.receive_packets().await;
            if !packets.is_empty() {
                // Store incoming packets in local queue for processing
                let mut queue = self.packet_queue.write().await;
                for packet in packets {
                    queue.enqueue_incoming(
                        packet.channel_id,
                        packet.packet_type,
                        packet.payload.to_vec(),
                    ).await?;
                }
            }
        }
        
        // Process outgoing packet queue (for batching if needed)
        let mut queue = self.packet_queue.write().await;
        let batched_packets = queue.flush_frame().await?;
        
        // Send any remaining batched packets
        if let Some(ws) = &self.ws_client {
            use playground_server::packet::Packet;
            use bytes::Bytes;
            
            for (channel, packets) in batched_packets {
                for outgoing in packets {
                    let packet = Packet {
                        channel_id: channel,
                        packet_type: outgoing.packet_type,
                        priority: match outgoing.priority {
                            0 => playground_server::packet::Priority::Low,
                            1 => playground_server::packet::Priority::Medium,
                            2 => playground_server::packet::Priority::High,
                            3 => playground_server::packet::Priority::Critical,
                            4 => playground_server::packet::Priority::Blocker,
                            _ => playground_server::packet::Priority::Medium,
                        },
                        payload: Bytes::from(outgoing.data),
                    };
                    ws.send_packet(packet).await?;
                }
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