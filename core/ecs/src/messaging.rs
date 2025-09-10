//! Messaging contracts for the ECS
//! 
//! This defines the contracts for the messaging system that is fundamental
//! to the ECS. All systems and components can use messaging for communication.

use bytes::Bytes;
use async_trait::async_trait;
use crate::error::EcsResult;

/// Channel ID type for message routing
pub type ChannelId = u16;

/// Trait for message handlers
/// This is the contract that message handlers must implement
#[async_trait]
pub trait MessageHandlerData: Send + Sync + 'static {
    /// Get unique handler ID
    fn handler_id(&self) -> String;
    
    /// Handle a message
    async fn handle(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
    
    /// Serialize the handler configuration
    async fn serialize(&self) -> EcsResult<Bytes>;
    
    /// Deserialize handler configuration
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> where Self: Sized;
}

/// Trait for message broadcasters
#[async_trait]
pub trait BroadcasterData: Send + Sync + 'static {
    /// Get unique broadcaster ID
    fn broadcaster_id(&self) -> String;
    
    /// Broadcast a message to all subscribers on a channel
    async fn broadcast(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
    
    /// Serialize the broadcaster configuration
    async fn serialize(&self) -> EcsResult<Bytes>;
    
    /// Deserialize broadcaster configuration
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> where Self: Sized;
}

/// Trait for the message bus
/// This is the core messaging infrastructure of the ECS
#[async_trait]
pub trait MessageBusContract: Send + Sync {
    /// Publish a message to a channel
    async fn publish(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
    
    /// Subscribe to a channel
    async fn subscribe(&self, channel: ChannelId, handler_id: String) -> EcsResult<()>;
    
    /// Unsubscribe from a channel
    async fn unsubscribe(&self, channel: ChannelId, handler_id: &str) -> EcsResult<()>;
    
    /// Check if a channel has subscribers
    async fn has_subscribers(&self, channel: ChannelId) -> bool;
    
    /// Get all active channels
    async fn get_channels(&self) -> Vec<ChannelId>;
    
    /// Clear all subscriptions for a channel
    async fn clear_channel(&self, channel: ChannelId) -> EcsResult<()>;
}