use std::collections::HashMap;
use async_trait::async_trait;
use playground_core_server::{ChannelManagerContract, ChannelManifest};
use playground_core_types::{Shared, shared};

pub struct ChannelManager {
    channels: Shared<HashMap<u16, String>>,
    name_to_channel: Shared<HashMap<String, u16>>,
}

impl ChannelManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut manager = Self {
            channels: shared(HashMap::new()),
            name_to_channel: shared(HashMap::new()),
        };
        
        // Register default control channel
        manager.register(0, "control".to_string()).await?;
        
        Ok(manager)
    }
}

#[async_trait]
impl ChannelManagerContract for ChannelManager {
    async fn register(&self, channel: u16, name: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut channels = self.channels.write().await;
        let mut name_map = self.name_to_channel.write().await;
        
        if channels.contains_key(&channel) {
            return Err(format!("Channel {} already registered", channel).into());
        }
        
        if name_map.contains_key(&name) {
            return Err(format!("Channel name '{}' already in use", name).into());
        }
        
        channels.insert(channel, name.clone());
        name_map.insert(name, channel);
        
        Ok(())
    }
    
    async fn unregister(&self, channel: u16) -> Result<(), Box<dyn std::error::Error>> {
        let mut channels = self.channels.write().await;
        let mut name_map = self.name_to_channel.write().await;
        
        if let Some(name) = channels.remove(&channel) {
            name_map.remove(&name);
            Ok(())
        } else {
            Err(format!("Channel {} not found", channel).into())
        }
    }
    
    async fn get_manifest(&self) -> ChannelManifest {
        let channels = self.channels.read().await;
        let mut manifest = ChannelManifest::new();
        
        for (channel_id, name) in channels.iter() {
            manifest.channels.insert(name.clone(), *channel_id);
        }
        
        manifest
    }
    
    async fn get_channel_by_name(&self, name: &str) -> Option<u16> {
        let name_map = self.name_to_channel.read().await;
        name_map.get(name).copied()
    }
    
    async fn is_registered(&self, channel: u16) -> bool {
        let channels = self.channels.read().await;
        channels.contains_key(&channel)
    }
}