//! Generic message handling contracts

use async_trait::async_trait;
use crate::types::*;
use playground_core_types::CoreError;

/// Contract for handling messages
#[async_trait]
pub trait MessageContract: Send + Sync {
    /// Create a new message
    fn create(&self, channel: ChannelId, priority: MessagePriority, payload: Vec<u8>) -> Message;
    
    /// Validate a message
    async fn validate(&self, message: &Message) -> Result<(), CoreError>;
    
    /// Serialize a message for transport
    async fn serialize(&self, message: &Message) -> Result<Vec<u8>, CoreError>;
    
    /// Deserialize a message from transport
    async fn deserialize(&self, data: &[u8]) -> Result<Message, CoreError>;
}

/// Contract for message handlers
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    async fn handle(&self, connection: ConnectionId, message: Message) -> Result<(), CoreError>;
    
    /// Get the channels this handler is interested in
    async fn channels(&self) -> Vec<ChannelId>;
}