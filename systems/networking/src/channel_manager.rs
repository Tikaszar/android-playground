//! Channel management for dynamic registration with core/server

use crate::{NetworkError, NetworkResult};
use playground_core_types::ChannelId;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU16, Ordering};

/// Manages channel registration and allocation
pub struct ChannelManager {
    // Map of channel names to IDs
    channels: DashMap<String, ChannelId>,
    // Map of IDs to names for reverse lookup
    id_to_name: DashMap<ChannelId, String>,
    // Next available plugin channel ID (starts at 1000)
    next_plugin_channel: AtomicU16,
    // Next available system channel ID (starts at 1)
    next_system_channel: AtomicU16,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: DashMap::new(),
            id_to_name: DashMap::new(),
            next_plugin_channel: AtomicU16::new(1000),
            next_system_channel: AtomicU16::new(1),
        }
    }
    
    /// Register a system channel (1-999)
    pub async fn register_system_channel(&mut self, name: &str, preferred_id: u16) -> NetworkResult<ChannelId> {
        if preferred_id == 0 || preferred_id >= 1000 {
            return Err(NetworkError::ChannelError(
                format!("System channels must be in range 1-999, got {}", preferred_id)
            ));
        }
        
        // Check if name already registered
        if self.channels.contains_key(name) {
            return self.channels.get(name)
                .map(|entry| *entry.value())
                .ok_or_else(|| NetworkError::ChannelError("Race condition".to_string()));
        }
        
        // Check if ID already taken
        if self.id_to_name.contains_key(&preferred_id) {
            // Find next available ID
            let mut id = self.next_system_channel.load(Ordering::SeqCst);
            while id < 1000 && self.id_to_name.contains_key(&id) {
                id += 1;
            }
            
            if id >= 1000 {
                return Err(NetworkError::ChannelError(
                    "No available system channels".to_string()
                ));
            }
            
            self.next_system_channel.store(id + 1, Ordering::SeqCst);
            self.channels.insert(name.to_string(), id);
            self.id_to_name.insert(id, name.to_string());
            Ok(id)
        } else {
            // Use preferred ID
            self.channels.insert(name.to_string(), preferred_id);
            self.id_to_name.insert(preferred_id, name.to_string());
            
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
        if let Some(id) = self.channels.get(name) {
            return Ok(*id.value());
        }
        
        // Allocate new channel ID
        let id = self.next_plugin_channel.fetch_add(1, Ordering::SeqCst);
        
        // Ensure we don't overflow
        if id > 65000 {
            return Err(NetworkError::ChannelError(
                "Plugin channel ID overflow".to_string()
            ));
        }
        
        self.channels.insert(name.to_string(), id);
        self.id_to_name.insert(id, name.to_string());
        
        Ok(id)
    }
    
    /// Look up a channel by name
    pub fn get_channel(&self, name: &str) -> Option<ChannelId> {
        self.channels.get(name).map(|entry| *entry.value())
    }
    
    /// Look up a channel name by ID
    pub fn get_channel_name(&self, id: ChannelId) -> Option<String> {
        self.id_to_name.get(&id).map(|entry| entry.value().clone())
    }
    
    /// Get total number of registered channels
    pub fn count(&self) -> usize {
        self.channels.len()
    }
    
    /// Check if a channel ID is for a system (< 1000) or plugin (>= 1000)
    pub fn is_system_channel(&self, id: ChannelId) -> bool {
        id > 0 && id < 1000
    }
    
    /// List all registered channels
    pub fn list_channels(&self) -> Vec<(String, ChannelId)> {
        self.channels
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect()
    }
}