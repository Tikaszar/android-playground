use crate::messages::{MessageEnvelope, PluginMessage};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender, Receiver};
use tracing::{debug, warn};
use uuid::Uuid;

/// Plugin handle for message bus registration
pub struct PluginHandle {
    pub id: Uuid,
    pub name: String,
    pub sender: Sender<MessageEnvelope>,
}

/// Central message bus for inter-plugin communication
pub struct MessageBus {
    /// Registered plugins
    plugins: Arc<DashMap<Uuid, PluginHandle>>,
    /// Message history for debugging
    history: Arc<DashMap<Uuid, MessageEnvelope>>,
    /// Global message channel
    broadcast_sender: Sender<MessageEnvelope>,
}

impl MessageBus {
    pub fn new() -> (Self, Receiver<MessageEnvelope>) {
        let (broadcast_sender, broadcast_receiver) = mpsc::channel(1000);
        
        (
            Self {
                plugins: Arc::new(DashMap::new()),
                history: Arc::new(DashMap::new()),
                broadcast_sender,
            },
            broadcast_receiver,
        )
    }
    
    /// Register a plugin with the message bus
    pub fn register_plugin(&self, id: Uuid, name: String) -> Receiver<MessageEnvelope> {
        let (sender, receiver) = mpsc::channel(100);
        
        let handle = PluginHandle {
            id,
            name: name.clone(),
            sender,
        };
        
        self.plugins.insert(id, handle);
        debug!("Registered plugin '{}' with ID {}", name, id);
        
        receiver
    }
    
    /// Unregister a plugin
    pub fn unregister_plugin(&self, id: Uuid) {
        if let Some((_, handle)) = self.plugins.remove(&id) {
            debug!("Unregistered plugin '{}' with ID {}", handle.name, id);
        }
    }
    
    /// Send a message
    pub async fn send(&self, envelope: MessageEnvelope) {
        // Store in history
        self.history.insert(envelope.id, envelope.clone());
        
        // Route message
        match envelope.to {
            Some(target_id) => {
                // Direct message to specific plugin
                if let Some(handle) = self.plugins.get(&target_id) {
                    if let Err(e) = handle.sender.send(envelope.clone()).await {
                        warn!("Failed to send message to plugin {}: {}", target_id, e);
                    }
                } else {
                    warn!("Target plugin {} not found", target_id);
                }
            }
            None => {
                // Broadcast to all plugins except sender
                for entry in self.plugins.iter() {
                    if entry.key() != &envelope.from {
                        if let Err(e) = entry.value().sender.send(envelope.clone()).await {
                            warn!("Failed to broadcast to plugin {}: {}", entry.key(), e);
                        }
                    }
                }
                
                // Also send to global broadcast channel
                let _ = self.broadcast_sender.send(envelope).await;
            }
        }
    }
    
    /// Create a message envelope
    pub fn create_envelope(from: Uuid, to: Option<Uuid>, message: PluginMessage) -> MessageEnvelope {
        MessageEnvelope {
            id: Uuid::new_v4(),
            from,
            to,
            message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Get registered plugins
    pub fn get_plugins(&self) -> Vec<(Uuid, String)> {
        self.plugins
            .iter()
            .map(|entry| (*entry.key(), entry.value().name.clone()))
            .collect()
    }
    
    /// Get message history
    pub fn get_history(&self, limit: usize) -> Vec<MessageEnvelope> {
        let mut history: Vec<_> = self.history
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        history.sort_by_key(|m| m.timestamp);
        history.into_iter().rev().take(limit).collect()
    }
}