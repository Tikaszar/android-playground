//! WebSocket contract for network communication
//!
//! This defines the contract for WebSocket handling that integrates with ECS messaging.
//! WebSocket IS a message handler in the unified system.

use async_trait::async_trait;
use crate::types::{Packet, ConnectionHandle};
use playground_core_ecs::MessageHandlerData;

/// Contract for WebSocket handling that integrates with ECS messaging
/// WebSocket IS a message handler in the unified system
#[async_trait]
pub trait WebSocketContract: Send + Sync + MessageHandlerData {
    /// Add a new WebSocket connection
    async fn add_connection(&self, conn: ConnectionHandle) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Remove a WebSocket connection
    async fn remove_connection(&self, id: usize) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Get current connection count
    async fn connection_count(&self) -> usize;
    
    /// Broadcast packet to all connections
    async fn broadcast(&self, packet: Packet) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Send packet to specific connection
    async fn send_to(&self, conn_id: usize, packet: Packet) -> Result<(), Box<dyn std::error::Error>>;
}