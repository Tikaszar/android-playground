//! Messaging system for the ECS
//! 
//! Provides pub/sub messaging between entities and systems using concrete types.

use std::collections::{HashMap, HashSet};
use bytes::Bytes;
use tokio::sync::mpsc;
use playground_core_types::{Shared, shared};
use crate::{CoreResult, CoreError};

/// Channel ID type for message routing
pub type ChannelId = u16;

/// Message to be sent through the bus
#[derive(Debug, Clone)]
pub struct Message {
    pub channel: ChannelId,
    pub data: Bytes,
    pub sender: String,
}

/// Subscription to a channel
#[derive(Debug, Clone)]
pub struct Subscription {
    pub channel: ChannelId,
    pub handler_id: String,
    pub sender: mpsc::Sender<Message>,
}

/// Concrete message bus implementation
pub struct MessageBus {
    /// Subscriptions: channel -> set of handler IDs
    subscriptions: Shared<HashMap<ChannelId, HashSet<String>>>,
    
    /// Handlers: handler_id -> channel for sending messages
    handlers: Shared<HashMap<String, mpsc::Sender<Message>>>,
    
    /// Active channels
    channels: Shared<HashSet<ChannelId>>,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self {
            subscriptions: shared(HashMap::new()),
            handlers: shared(HashMap::new()),
            channels: shared(HashSet::new()),
        }
    }
    
    /// Publish a message to a channel
    pub async fn publish(&self, channel: ChannelId, data: Bytes, sender: String) -> CoreResult<()> {
        let message = Message { channel, data, sender };
        
        // Get all handlers subscribed to this channel
        let handlers = {
            let subs = self.subscriptions.read().await;
            let handler_map = self.handlers.read().await;
            
            if let Some(handler_ids) = subs.get(&channel) {
                let mut handlers = Vec::new();
                for handler_id in handler_ids {
                    if let Some(sender) = handler_map.get(handler_id) {
                        handlers.push(sender.clone());
                    }
                }
                handlers
            } else {
                return Ok(()); // No subscribers
            }
        };
        
        // Send message to all handlers
        for handler in handlers {
            // We ignore send errors as handlers may have disconnected
            let _ = handler.send(message.clone()).await;
        }
        
        Ok(())
    }
    
    /// Subscribe to a channel
    pub async fn subscribe(&self, channel: ChannelId, handler_id: String) -> CoreResult<mpsc::Receiver<Message>> {
        // Create channel for this handler
        let (tx, rx) = mpsc::channel(100);
        
        // Add to subscriptions
        let mut subs = self.subscriptions.write().await;
        subs.entry(channel)
            .or_insert_with(HashSet::new)
            .insert(handler_id.clone());
        
        // Add to handlers
        let mut handlers = self.handlers.write().await;
        handlers.insert(handler_id, tx);
        
        // Track channel as active
        let mut channels = self.channels.write().await;
        channels.insert(channel);
        
        Ok(rx)
    }
    
    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: ChannelId, handler_id: &str) -> CoreResult<()> {
        // Remove from subscriptions
        let mut subs = self.subscriptions.write().await;
        if let Some(handlers) = subs.get_mut(&channel) {
            handlers.remove(handler_id);
            
            // Remove channel if no more subscribers
            if handlers.is_empty() {
                subs.remove(&channel);
                
                let mut channels = self.channels.write().await;
                channels.remove(&channel);
            }
        }
        
        // Remove handler
        let mut handlers = self.handlers.write().await;
        handlers.remove(handler_id);
        
        Ok(())
    }
    
    /// Check if a channel has subscribers
    pub async fn has_subscribers(&self, channel: ChannelId) -> bool {
        let subs = self.subscriptions.read().await;
        subs.get(&channel)
            .map(|handlers| !handlers.is_empty())
            .unwrap_or(false)
    }
    
    /// Get all active channels
    pub async fn get_channels(&self) -> Vec<ChannelId> {
        let channels = self.channels.read().await;
        channels.iter().copied().collect()
    }
    
    /// Clear all subscriptions for a channel
    pub async fn clear_channel(&self, channel: ChannelId) -> CoreResult<()> {
        let mut subs = self.subscriptions.write().await;
        let handler_ids: Vec<String> = subs.get(&channel)
            .map(|handlers| handlers.iter().cloned().collect())
            .unwrap_or_default();
        
        // Remove channel from subscriptions
        subs.remove(&channel);
        
        // Remove all handlers for this channel
        let mut handlers = self.handlers.write().await;
        for handler_id in handler_ids {
            handlers.remove(&handler_id);
        }
        
        // Remove from active channels
        let mut channels = self.channels.write().await;
        channels.remove(&channel);
        
        Ok(())
    }
    
    /// Get subscriber count for a channel
    pub async fn subscriber_count(&self, channel: ChannelId) -> usize {
        let subs = self.subscriptions.read().await;
        subs.get(&channel)
            .map(|handlers| handlers.len())
            .unwrap_or(0)
    }
}