//! Generic message handling contracts

use async_trait::async_trait;
use crate::types::*;
use std::error::Error;

/// Contract for handling messages
#[async_trait]
pub trait MessageContract: Send + Sync {
    /// Create a new message
    fn create(&self, channel: ChannelId, priority: MessagePriority, payload: Vec<u8>) -> Message;
    
    /// Validate a message
    async fn validate(&self, message: &Message) -> Result<(), Box<dyn Error>>;
    
    /// Serialize a message for transport
    async fn serialize(&self, message: &Message) -> Result<Vec<u8>, Box<dyn Error>>;
    
    /// Deserialize a message from transport
    async fn deserialize(&self, data: &[u8]) -> Result<Message, Box<dyn Error>>;
}

/// Contract for message handlers
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    async fn handle(&self, connection: ConnectionId, message: Message) -> Result<(), Box<dyn Error>>;
    
    /// Get the channels this handler is interested in
    async fn channels(&self) -> Vec<ChannelId>;
}