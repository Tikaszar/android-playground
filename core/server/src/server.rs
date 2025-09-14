//! Generic server contract
//!
//! This defines a generic server that can be implemented by any transport
//! (WebSocket, TCP, UDP, IPC, named pipes, etc.)

use async_trait::async_trait;
use crate::types::*;
use playground_core_types::{CoreError, CoreResult};

/// Generic contract for any server implementation
#[async_trait]
pub trait ServerContract: Send + Sync {
    /// Start the server
    async fn start(&self, config: ServerConfig) -> CoreResult<()>;
    
    /// Stop the server gracefully
    async fn stop(&self) -> CoreResult<()>;
    
    /// Check if the server is running
    async fn is_running(&self) -> bool;
    
    /// Get server statistics
    async fn stats(&self) -> ServerStats;
    
    /// Get server configuration
    async fn config(&self) -> ServerConfig;
    
    /// Handle incoming connection (called by implementation)
    async fn on_connection(&self, connection: ConnectionInfo) -> CoreResult<()>;
    
    /// Handle connection closed (called by implementation)
    async fn on_disconnection(&self, id: ConnectionId) -> CoreResult<()>;
    
    /// Send a message to a specific connection
    async fn send_to(&self, connection: ConnectionId, message: Message) -> CoreResult<()>;
    
    /// Broadcast a message to all connections
    async fn broadcast(&self, message: Message) -> CoreResult<()>;
    
    /// Send a message to all connections on a channel
    async fn publish(&self, channel: ChannelId, message: Message) -> CoreResult<()>;
    
    /// Get list of active connections
    async fn connections(&self) -> Vec<ConnectionInfo>;
    
    /// Get connection info
    async fn connection(&self, id: ConnectionId) -> Option<ConnectionInfo>;
}