//! Messaging implementation for the unified ECS
//! 
//! This is core ECS functionality that allows all systems and components
//! to communicate via message passing.

use std::collections::HashMap;
use bytes::Bytes;
use async_trait::async_trait;
use tokio::sync::mpsc;
use playground_core_types::{Shared, shared};
use playground_core_ecs::{
    ChannelId, MessageHandlerData, BroadcasterData, MessageBusContract,
    EcsResult, EcsError
};

/// Concrete message handler wrapper (NO dyn pattern)
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

/// Concrete broadcaster wrapper (NO dyn pattern)
#[derive(Clone)]
pub struct BroadcasterWrapper {
    broadcaster_id: String,
    broadcaster_type: String,
    sender: Option<mpsc::UnboundedSender<(ChannelId, Bytes)>>,
}

impl BroadcasterWrapper {
    /// Create a new broadcaster from data
    pub async fn new<B: BroadcasterData>(broadcaster: B) -> EcsResult<Self> {
        Ok(Self {
            broadcaster_id: broadcaster.broadcaster_id(),
            broadcaster_type: std::any::type_name::<B>().to_string(),
            sender: None,
        })
    }
    
    /// Create a channel-based broadcaster
    pub fn new_channel(broadcaster_id: String) -> (Self, mpsc::UnboundedReceiver<(ChannelId, Bytes)>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let broadcaster = Self {
            broadcaster_id,
            broadcaster_type: "channel".to_string(),
            sender: Some(tx),
        };
        (broadcaster, rx)
    }
    
    /// Broadcast a message
    pub async fn broadcast(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
        if let Some(sender) = &self.sender {
            sender.send((channel, message))
                .map_err(|_| EcsError::MessageError("Broadcaster channel closed".to_string()))?;
        }
        Ok(())
    }
}

/// Message bus implementation for the ECS
/// This is the core messaging infrastructure
pub struct MessageBus {
    /// Subscribers for each channel
    subscribers: Shared<HashMap<ChannelId, Vec<MessageHandler>>>,
    /// Broadcasters for channels
    broadcasters: Shared<HashMap<ChannelId, Vec<BroadcasterWrapper>>>,
    /// Message queue for async processing
    message_queue: Shared<Vec<(ChannelId, Bytes)>>,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self {
            subscribers: shared(HashMap::new()),
            broadcasters: shared(HashMap::new()),
            message_queue: shared(Vec::new()),
        }
    }
    
    /// Subscribe with a MessageHandler directly (for compatibility)
    pub async fn subscribe_with_handler(&self, channel: ChannelId, handler: MessageHandler) -> EcsResult<()> {
        self.subscribe_handler(channel, handler).await
    }
    
    /// Subscribe a handler to a channel
    pub async fn subscribe_handler(&self, channel: ChannelId, handler: MessageHandler) -> EcsResult<()> {
        let mut subs = self.subscribers.write().await;
        subs.entry(channel)
            .or_insert_with(Vec::new)
            .push(handler);
        Ok(())
    }
    
    /// Register a broadcaster for a channel
    pub async fn register_broadcaster(&self, channel: ChannelId, broadcaster: BroadcasterWrapper) -> EcsResult<()> {
        let mut broadcasters = self.broadcasters.write().await;
        broadcasters.entry(channel)
            .or_insert_with(Vec::new)
            .push(broadcaster);
        Ok(())
    }
    
    /// Process queued messages
    pub async fn process_queue(&self) -> EcsResult<()> {
        let messages = {
            let mut queue = self.message_queue.write().await;
            std::mem::take(&mut *queue)
        };
        
        for (channel, message) in messages {
            self.deliver_message(channel, message).await?;
        }
        
        Ok(())
    }
    
    /// Deliver a message to all subscribers
    async fn deliver_message(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
        let handlers = {
            let subs = self.subscribers.read().await;
            subs.get(&channel).cloned().unwrap_or_default()
        };
        
        for handler in handlers {
            handler.handle(channel, message.clone()).await?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl MessageBusContract for MessageBus {
    async fn publish(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
        // Queue the message for processing
        self.message_queue.write().await.push((channel, message.clone()));
        
        // Also deliver immediately for low latency
        self.deliver_message(channel, message).await
    }
    
    async fn subscribe(&self, channel: ChannelId, handler_id: String) -> EcsResult<()> {
        let (handler, _rx) = MessageHandler::new_channel(handler_id);
        self.subscribe_handler(channel, handler).await
    }
    
    async fn unsubscribe(&self, channel: ChannelId, handler_id: &str) -> EcsResult<()> {
        let mut subs = self.subscribers.write().await;
        if let Some(handlers) = subs.get_mut(&channel) {
            handlers.retain(|h| h.handler_id() != handler_id);
        }
        Ok(())
    }
    
    async fn has_subscribers(&self, channel: ChannelId) -> bool {
        let subs = self.subscribers.read().await;
        subs.get(&channel).map_or(false, |h| !h.is_empty())
    }
    
    async fn get_channels(&self) -> Vec<ChannelId> {
        let subs = self.subscribers.read().await;
        subs.keys().cloned().collect()
    }
    
    async fn clear_channel(&self, channel: ChannelId) -> EcsResult<()> {
        self.subscribers.write().await.remove(&channel);
        self.broadcasters.write().await.remove(&channel);
        Ok(())
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}