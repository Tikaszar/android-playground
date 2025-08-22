//! Channel management for dynamic registration with core/server

use crate::{NetworkError, NetworkResult};
use playground_core_types::{ChannelId, Shared, shared};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU16, Ordering};
use bytes::Bytes;

/// Manages channel registration and allocation
pub struct ChannelManager {
    // Map of channel names to IDs
    channels: Shared<HashMap<String, ChannelId>>,
    // Map of IDs to names for reverse lookup
    id_to_name: Shared<HashMap<ChannelId, String>>,
    // Next available plugin channel ID (starts at 1000)
    next_plugin_channel: AtomicU16,
    // Next available system channel ID (starts at 1)
    next_system_channel: AtomicU16,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: shared(HashMap::new()),
            id_to_name: shared(HashMap::new()),
            next_plugin_channel: AtomicU16::new(1000),
            next_system_channel: AtomicU16::new(1),
        }
    }
    
    /// Register a channel with a specific ID
    pub async fn register_channel(&mut self, channel_id: u16, name: String) -> NetworkResult<()> {
        // Check if ID already taken
        if self.id_to_name.read().await.contains_key(&channel_id) {
            let existing_name = self.id_to_name.read().await.get(&channel_id).cloned();
            if let Some(existing) = existing_name {
                if existing != name {
                    return Err(NetworkError::ChannelError(
                        format!("Channel {} already registered as '{}'", channel_id, existing)
                    ));
                }
            }
            // Already registered with same name, that's OK
            return Ok(());
        }
        
        self.channels.write().await.insert(name.clone(), channel_id);
        self.id_to_name.write().await.insert(channel_id, name);
        Ok(())
    }
    
    /// Register a system channel (1-999)
    pub async fn register_system_channel(&mut self, name: &str, preferred_id: u16) -> NetworkResult<ChannelId> {
        if preferred_id == 0 || preferred_id >= 1000 {
            return Err(NetworkError::ChannelError(
                format!("System channels must be in range 1-999, got {}", preferred_id)
            ));
        }
        
        // Check if name already registered
        if self.channels.read().await.contains_key(name) {
            return self.channels.read().await.get(name)
                .copied()
                .ok_or_else(|| NetworkError::ChannelError("Race condition".to_string()));
        }
        
        // Check if ID already taken
        if self.id_to_name.read().await.contains_key(&preferred_id) {
            // Find next available ID
            let mut id = self.next_system_channel.load(Ordering::SeqCst);
            while id < 1000 && self.id_to_name.read().await.contains_key(&id) {
                id += 1;
            }
            
            if id >= 1000 {
                return Err(NetworkError::ChannelError(
                    "No available system channels".to_string()
                ));
            }
            
            self.next_system_channel.store(id + 1, Ordering::SeqCst);
            self.channels.write().await.insert(name.to_string(), id);
            self.id_to_name.write().await.insert(id, name.to_string());
            Ok(id)
        } else {
            // Use preferred ID
            self.channels.write().await.insert(name.to_string(), preferred_id);
            self.id_to_name.write().await.insert(preferred_id, name.to_string());
            
            // Update next available if needed
            let current_next = self.next_system_channel.load(Ordering::SeqCst);
            if preferred_id >= current_next {
                self.next_system_channel.store(preferred_id + 1, Ordering::SeqCst);
            }
            
            Ok(preferred_id)
        }
    }
    
    /// Register a plugin channel (1000+)
    pub async fn register_plugin_channel(&mut self, name: &str) -> NetworkResult<ChannelId> {
        // Check if already registered
        if let Some(&id) = self.channels.read().await.get(name) {
            return Ok(id);
        }
        
        // Allocate new channel ID
        let id = self.next_plugin_channel.fetch_add(1, Ordering::SeqCst);
        
        // Ensure we don't overflow
        if id > 65000 {
            return Err(NetworkError::ChannelError(
                "Plugin channel ID overflow".to_string()
            ));
        }
        
        self.channels.write().await.insert(name.to_string(), id);
        self.id_to_name.write().await.insert(id, name.to_string());
        
        Ok(id)
    }
    
    /// Look up a channel by name
    pub async fn get_channel(&self, name: &str) -> Option<ChannelId> {
        self.channels.read().await.get(name).copied()
    }
    
    /// Look up a channel name by ID
    pub async fn get_channel_name(&self, id: ChannelId) -> Option<String> {
        self.id_to_name.read().await.get(&id).cloned()
    }
    
    /// Get total number of registered channels
    pub async fn count(&self) -> usize {
        self.channels.read().await.len()
    }
    
    /// Check if a channel ID is for a system (< 1000) or plugin (>= 1000)
    pub fn is_system_channel(&self, id: ChannelId) -> bool {
        id > 0 && id < 1000
    }
    
    /// List all registered channels
    pub async fn list_channels(&self) -> Vec<(String, ChannelId)> {
        self.channels
            .read()
            .await
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}