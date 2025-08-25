use std::collections::HashMap;
use bytes::Bytes;
use playground_core_types::{Handle, Shared, shared};
use crate::error::{LogicResult, LogicError};

/// Channel ID type for message routing
pub type ChannelId = u16;

/// Concrete wrapper for message handlers (similar to Component pattern)
pub struct MessageHandlerData {
    data: Bytes,
    handler_name: String,
    plugin_name: String,  // Which plugin/system owns this handler
}

impl MessageHandlerData {
    /// Create a new message handler with serialized configuration
    pub fn new(plugin_name: String, handler_name: String, config_data: Bytes) -> Self {
        Self {
            data: config_data,
            handler_name,
            plugin_name,
        }
    }
    
    /// Get the handler name
    pub fn handler_name(&self) -> &str {
        &self.handler_name
    }
    
    /// Get the plugin name
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }
    
    /// Get the configuration data
    pub fn data(&self) -> &Bytes {
        &self.data
    }
    
    /// Handle a message - actual implementation would be in the plugin/system
    pub async fn handle(&self, _channel: ChannelId, _message: Bytes) -> LogicResult<()> {
        // The actual handling logic would be implemented by looking up
        // the handler by plugin_name and handler_name in a registry
        // For now, this is a placeholder
        Ok(())
    }
}

/// Game message bus for plugin/app communication
pub struct GameMessageBus {
    subscribers: Shared<HashMap<ChannelId, Vec<MessageHandlerData>>>,
    // Bridge to core/ecs message bus for system-level messaging
    core_bus: Option<Handle<playground_core_ecs::MessageBus>>,
}

impl GameMessageBus {
    /// Create a new game message bus
    pub fn new() -> Self {
        Self {
            subscribers: shared(HashMap::new()),
            core_bus: None,
        }
    }
    
    /// Set the core message bus for bridging
    pub fn set_core_bus(&mut self, core_bus: Handle<playground_core_ecs::MessageBus>) {
        self.core_bus = Some(core_bus);
    }
    
    /// Subscribe to a channel
    pub async fn subscribe(
        &self,
        channel: ChannelId,
        handler: MessageHandlerData,
    ) -> LogicResult<()> {
        let mut subs = self.subscribers.write().await;
        subs.entry(channel)
            .or_insert_with(Vec::new)
            .push(handler);
        Ok(())
    }
    
    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: ChannelId) -> LogicResult<()> {
        let mut subs = self.subscribers.write().await;
        subs.remove(&channel);
        Ok(())
    }
    
    /// Unsubscribe a specific handler from a channel
    pub async fn unsubscribe_handler(&self, channel: ChannelId, handler_name: &str) -> LogicResult<()> {
        let mut subs = self.subscribers.write().await;
        if let Some(handlers) = subs.get_mut(&channel) {
            handlers.retain(|h| h.handler_name() != handler_name);
        }
        Ok(())
    }
    
    /// Publish a message to a channel
    pub async fn publish(&self, channel: ChannelId, message: Bytes) -> LogicResult<()> {
        // First, notify all game-level subscribers
        let subs = self.subscribers.read().await;
        if let Some(handlers) = subs.get(&channel) {
            for handler in handlers {
                handler.handle(channel, message.clone()).await?;
            }
        }
        
        // Then, forward to core message bus if configured
        if let Some(core_bus) = &self.core_bus {
            core_bus.publish(channel, message).await
                .map_err(|e| LogicError::SystemError(format!("Core bus error: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Get subscriber count for a channel
    pub async fn subscriber_count(&self, channel: ChannelId) -> usize {
        let subs = self.subscribers.read().await;
        subs.get(&channel).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for GameMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Channel ranges for game systems
pub mod channels {
    /// System channels (1-999)
    pub const SYSTEM_CHANNEL_START: u16 = 1;
    pub const SYSTEM_CHANNEL_END: u16 = 999;
    
    /// Plugin channels (1000-1999)
    pub const PLUGIN_CHANNEL_START: u16 = 1000;
    pub const PLUGIN_CHANNEL_END: u16 = 1999;
    
    /// Browser client channels (2000-2999)
    pub const BROWSER_CHANNEL_START: u16 = 2000;
    pub const BROWSER_CHANNEL_END: u16 = 2999;
    
    /// MCP session channels (3000-3999)
    pub const MCP_CHANNEL_START: u16 = 3000;
    pub const MCP_CHANNEL_END: u16 = 3999;
    
    /// Well-known system channels
    pub const UI_RENDER_CHANNEL: u16 = 10;
    pub const NETWORKING_CHANNEL: u16 = 100;
    pub const PHYSICS_CHANNEL: u16 = 200;
    pub const AUDIO_CHANNEL: u16 = 300;
}