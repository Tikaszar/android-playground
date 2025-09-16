//! Generic channel contract for message routing
//!
//! Channels provide logical grouping of messages, similar to topics or rooms

use async_trait::async_trait;
use crate::types::*;
use playground_core_types::CoreError;

/// Generic contract for channel management
#[async_trait]
pub trait ChannelContract: Send + Sync {
    /// Create a new channel
    async fn create(&self, name: String, description: Option<String>) -> Result<ChannelId, CoreError>;
    
    /// Delete a channel
    async fn delete(&self, id: ChannelId) -> Result<(), CoreError>;
    
    /// Get channel info
    async fn info(&self, id: ChannelId) -> Option<ChannelInfo>;
    
    /// List all channels
    async fn list(&self) -> Vec<ChannelInfo>;
    
    /// Subscribe a connection to a channel
    async fn subscribe(&self, channel: ChannelId, connection: ConnectionId) -> Result<(), CoreError>;
    
    /// Unsubscribe a connection from a channel
    async fn unsubscribe(&self, channel: ChannelId, connection: ConnectionId) -> Result<(), CoreError>;
    
    /// Get all subscribers for a channel
    async fn subscribers(&self, channel: ChannelId) -> Vec<ConnectionId>;
    
    /// Get all channels a connection is subscribed to
    async fn subscriptions(&self, connection: ConnectionId) -> Vec<ChannelId>;
    
    /// Publish a message to a channel (sent to all subscribers)
    async fn publish(&self, channel: ChannelId, message: Message) -> Result<(), CoreError>;
}