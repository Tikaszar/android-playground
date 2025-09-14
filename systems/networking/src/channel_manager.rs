use std::collections::{HashMap, HashSet};
use playground_core_server::ConnectionId;
use playground_core_types::{Shared, shared, CoreResult, CoreError};
use crate::types::ChannelManifest;

pub struct ChannelManager {
    channels: Shared<HashMap<u16, String>>,
    name_to_channel: Shared<HashMap<String, u16>>,
    subscriptions: Shared<HashMap<u16, HashSet<ConnectionId>>>,
}

impl ChannelManager {
    pub async fn new() -> CoreResult<Self> {
        let mut manager = Self {
            channels: shared(HashMap::new()),
            name_to_channel: shared(HashMap::new()),
            subscriptions: shared(HashMap::new()),
        };
        
        // Register default control channel
        manager.register(0, "control".to_string()).await?;
        
        Ok(manager)
    }
    pub async fn register(&self, channel: u16, name: String) -> CoreResult<()> {
        let mut channels = self.channels.write().await;
        let mut name_map = self.name_to_channel.write().await;
        
        if channels.contains_key(&channel) {
            return Err(CoreError::InvalidInput(format!("Channel {} already registered", channel)));
        }
        
        if name_map.contains_key(&name) {
            return Err(CoreError::InvalidInput(format!("Channel name '{}' already in use", name)));
        }
        
        channels.insert(channel, name.clone());
        name_map.insert(name, channel);
        
        Ok(())
    }
    
    pub async fn unregister(&self, channel: u16) -> CoreResult<()> {
        let mut channels = self.channels.write().await;
        let mut name_map = self.name_to_channel.write().await;
        
        if let Some(name) = channels.remove(&channel) {
            name_map.remove(&name);
            Ok(())
        } else {
            Err(CoreError::NotFound(format!("Channel {} not found", channel)))
        }
    }
    
    pub async fn get_manifest(&self) -> ChannelManifest {
        let channels = self.channels.read().await;
        let mut manifest = ChannelManifest::new();
        
        for (channel_id, name) in channels.iter() {
            manifest.channels.insert(name.clone(), *channel_id);
        }
        
        manifest
    }
    
    pub async fn get_channel_by_name(&self, name: &str) -> Option<u16> {
        let name_map = self.name_to_channel.read().await;
        name_map.get(name).copied()
    }
    
    pub async fn is_registered(&self, channel: u16) -> bool {
        let channels = self.channels.read().await;
        channels.contains_key(&channel)
    }
    
    pub async fn subscribe(&self, channel: u16, connection: ConnectionId) -> CoreResult<()> {
        let mut subs = self.subscriptions.write().await;
        subs.entry(channel)
            .or_insert_with(HashSet::new)
            .insert(connection);
        Ok(())
    }
    
    pub async fn unsubscribe(&self, channel: u16, connection: ConnectionId) -> CoreResult<()> {
        let mut subs = self.subscriptions.write().await;
        if let Some(connections) = subs.get_mut(&channel) {
            connections.remove(&connection);
        }
        Ok(())
    }
    
    pub async fn get_subscribers(&self, channel: u16) -> Vec<ConnectionId> {
        let subs = self.subscriptions.read().await;
        subs.get(&channel)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }
}