use std::collections::HashMap;
use bytes::Bytes;
use async_trait::async_trait;
use playground_core_types::{Shared, shared};
use tokio::sync::mpsc;
use crate::{EntityId, EcsResult, EcsError};

/// Channel ID type for message routing
pub type ChannelId = u16;

/// Message handler trait for actual handler implementations
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

/// Concrete wrapper for message handlers (NO dyn pattern)
#[derive(Clone)]
pub struct MessageHandler {
    handler_id: String,
    handler_type: String,
    // For handlers that can't be serialized, we use channels
    sender: Option<mpsc::UnboundedSender<(ChannelId, Bytes)>>,
}

impl MessageHandler {
    /// Create a new message handler from data
    pub async fn new<H: MessageHandlerData>(handler: H) -> EcsResult<Self> {
        Ok(Self {
            handler_id: handler.handler_id(),
            handler_type: std::any::type_name::<H>().to_string(),
            sender: None,
        })
    }
    
    /// Create a channel-based handler for runtime handlers
    pub fn new_channel(handler_id: String) -> (Self, mpsc::UnboundedReceiver<(ChannelId, Bytes)>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let handler = Self {
            handler_id,
            handler_type: "channel".to_string(),
            sender: Some(tx),
        };
        (handler, rx)
    }
    
    /// Get the handler ID
    pub fn handler_id(&self) -> &str {
        &self.handler_id
    }
    
    /// Send a message through the handler
    pub async fn handle(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
        if let Some(sender) = &self.sender {
            sender.send((channel, message))
                .map_err(|_| EcsError::MessageError("Handler channel closed".to_string()))?;
        }
        Ok(())
    }
}

/// Broadcaster trait for actual broadcaster implementations
#[async_trait]
pub trait BroadcasterData: Send + Sync + 'static {
    /// Get unique broadcaster ID
    fn broadcaster_id(&self) -> String;
    
    /// Broadcast a message to all subscribers on a channel
    async fn broadcast(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
    
    /// Send a message to a specific entity
    async fn send_to(&self, target: EntityId, message: Bytes) -> EcsResult<()>;
}

/// Concrete wrapper for broadcaster (NO dyn pattern)
#[derive(Clone)]
pub struct BroadcasterWrapper {
    broadcaster_id: String,
    // Use channels for runtime broadcasters
    sender: mpsc::UnboundedSender<BroadcastCommand>,
}

#[derive(Debug)]
pub enum BroadcastCommand {
    Broadcast(ChannelId, Bytes),
    SendTo(EntityId, Bytes),
}

impl BroadcasterWrapper {
    /// Create a channel-based broadcaster
    pub fn new(broadcaster_id: String) -> (Self, mpsc::UnboundedReceiver<BroadcastCommand>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let wrapper = Self {
            broadcaster_id,
            sender: tx,
        };
        (wrapper, rx)
    }
    
    /// Broadcast a message
    pub async fn broadcast(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
        self.sender.send(BroadcastCommand::Broadcast(channel, message))
            .map_err(|_| EcsError::MessageError("Broadcaster channel closed".to_string()))?;
        Ok(())
    }
    
    /// Send to entity
    pub async fn send_to(&self, target: EntityId, message: Bytes) -> EcsResult<()> {
        self.sender.send(BroadcastCommand::SendTo(target, message))
            .map_err(|_| EcsError::MessageError("Broadcaster channel closed".to_string()))?;
        Ok(())
    }
}

/// Internal message bus for ECS systems
pub struct MessageBus {
    subscribers: Shared<HashMap<ChannelId, Vec<MessageHandler>>>,
    entity_handlers: Shared<HashMap<EntityId, MessageHandler>>,
    broadcaster: Option<BroadcasterWrapper>,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self {
            subscribers: shared(HashMap::new()),
            entity_handlers: shared(HashMap::new()),
            broadcaster: None,
        }
    }
    
    /// Set an external broadcaster (e.g., for WebSocket forwarding)
    pub fn set_broadcaster(&mut self, broadcaster: BroadcasterWrapper) {
        self.broadcaster = Some(broadcaster);
    }
    
    /// Subscribe to a channel
    pub async fn subscribe(
        &self,
        channel: ChannelId,
        handler: MessageHandler,
    ) -> EcsResult<()> {
        let mut subs = self.subscribers.write().await;
        subs.entry(channel)
            .or_insert_with(Vec::new)
            .push(handler);
        Ok(())
    }
    
    /// Subscribe a handler for a specific entity
    pub async fn subscribe_entity(
        &self,
        entity: EntityId,
        handler: MessageHandler,
    ) -> EcsResult<()> {
        let mut handlers = self.entity_handlers.write().await;
        handlers.insert(entity, handler);
        Ok(())
    }
    
    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: ChannelId) -> EcsResult<()> {
        let mut subs = self.subscribers.write().await;
        subs.remove(&channel);
        Ok(())
    }
    
    /// Unsubscribe a specific handler from a channel
    pub async fn unsubscribe_handler(&self, channel: ChannelId, handler_id: &str) -> EcsResult<()> {
        let mut subs = self.subscribers.write().await;
        if let Some(handlers) = subs.get_mut(&channel) {
            handlers.retain(|h| h.handler_id() != handler_id);
        }
        Ok(())
    }
    
    /// Publish a message to a channel
    pub async fn publish(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
        // First, notify all internal subscribers
        let subs = self.subscribers.read().await;
        if let Some(handlers) = subs.get(&channel) {
            for handler in handlers {
                handler.handle(channel, message.clone()).await?;
            }
        }
        drop(subs); // Release lock before external broadcast
        
        // Then, forward to external broadcaster if configured
        if let Some(broadcaster) = &self.broadcaster {
            broadcaster.broadcast(channel, message).await?;
        }
        
        Ok(())
    }
    
    /// Send a message to a specific entity
    pub async fn send_to(&self, target: EntityId, message: Bytes) -> EcsResult<()> {
        let handlers = self.entity_handlers.read().await;
        if let Some(handler) = handlers.get(&target) {
            handler.handle(0, message.clone()).await?;
        }
        drop(handlers); // Release lock before external broadcast
        
        // Also forward to external broadcaster if configured
        if let Some(broadcaster) = &self.broadcaster {
            broadcaster.send_to(target, message).await?;
        }
        
        Ok(())
    }
    
    /// Get subscriber count for a channel
    pub async fn subscriber_count(&self, channel: ChannelId) -> usize {
        let subs = self.subscribers.read().await;
        subs.get(&channel).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}