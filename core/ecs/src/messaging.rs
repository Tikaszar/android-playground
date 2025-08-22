use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use bytes::Bytes;
use async_trait::async_trait;
use crate::{EntityId, EcsResult, EcsError};

/// Channel ID type for message routing
pub type ChannelId = u16;

/// Message handler trait for subscribers
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
}

/// Function-based message handler
pub struct FnHandler<F> {
    handler: F,
}

impl<F> FnHandler<F> {
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<F> MessageHandler for FnHandler<F>
where
    F: Fn(ChannelId, Bytes) -> EcsResult<()> + Send + Sync,
{
    async fn handle(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
        (self.handler)(channel, message)
    }
}

/// Broadcaster trait for sending messages
#[async_trait]
pub trait Broadcaster: Send + Sync {
    /// Broadcast a message to all subscribers on a channel
    async fn broadcast(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
    
    /// Send a message to a specific entity
    async fn send_to(&self, target: EntityId, message: Bytes) -> EcsResult<()>;
}

/// Internal message bus for ECS systems
pub struct MessageBus {
    subscribers: Arc<RwLock<HashMap<ChannelId, Vec<Arc<dyn MessageHandler>>>>>,
    entity_handlers: Arc<RwLock<HashMap<EntityId, Arc<dyn MessageHandler>>>>,
    broadcaster: Option<Arc<dyn Broadcaster>>,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            entity_handlers: Arc::new(RwLock::new(HashMap::new())),
            broadcaster: None,
        }
    }
    
    /// Set an external broadcaster (e.g., for WebSocket forwarding)
    pub async fn set_broadcaster(&mut self, broadcaster: Arc<dyn Broadcaster>) {
        self.broadcaster = Some(broadcaster);
    }
    
    /// Subscribe to a channel
    pub async fn subscribe(
        &self,
        channel: ChannelId,
        handler: Arc<dyn MessageHandler>,
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
        handler: Arc<dyn MessageHandler>,
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
    
    /// Publish a message to a channel
    pub async fn publish(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
        // First, notify all internal subscribers
        let subs = self.subscribers.read().await;
        if let Some(handlers) = subs.get(&channel) {
            for handler in handlers {
                handler.handle(channel, message.clone()).await?;
            }
        }
        
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