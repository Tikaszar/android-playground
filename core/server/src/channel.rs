use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, bail};

#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub id: u16,
    pub name: String,
    pub owner: String,
}

pub struct ChannelManager {
    channels: Arc<RwLock<HashMap<u16, ChannelInfo>>>,
    name_to_id: Arc<RwLock<HashMap<String, u16>>>,
    next_plugin_channel: Arc<RwLock<u16>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        let mut channels = HashMap::new();
        let mut name_to_id = HashMap::new();
        
        channels.insert(0, ChannelInfo {
            id: 0,
            name: "control".to_string(),
            owner: "core".to_string(),
        });
        name_to_id.insert("control".to_string(), 0);
        
        Self {
            channels: Arc::new(RwLock::new(channels)),
            name_to_id: Arc::new(RwLock::new(name_to_id)),
            next_plugin_channel: Arc::new(RwLock::new(1000)),
        }
    }
    
    pub async fn register_system(&self, name: String, channel_id: u16) -> Result<u16> {
        if channel_id == 0 || channel_id >= 1000 {
            bail!("System channels must be in range 1-999");
        }
        
        let mut channels = self.channels.write().await;
        let mut name_to_id = self.name_to_id.write().await;
        
        if channels.contains_key(&channel_id) {
            bail!("Channel {} already registered", channel_id);
        }
        
        if name_to_id.contains_key(&name) {
            bail!("Channel name '{}' already registered", name);
        }
        
        channels.insert(channel_id, ChannelInfo {
            id: channel_id,
            name: name.clone(),
            owner: "system".to_string(),
        });
        name_to_id.insert(name, channel_id);
        
        Ok(channel_id)
    }
    
    pub async fn register_plugin(&self, name: String) -> Result<u16> {
        let mut channels = self.channels.write().await;
        let mut name_to_id = self.name_to_id.write().await;
        
        if name_to_id.contains_key(&name) {
            bail!("Channel name '{}' already registered", name);
        }
        
        let mut next_id = self.next_plugin_channel.write().await;
        let channel_id = *next_id;
        *next_id += 1;
        
        channels.insert(channel_id, ChannelInfo {
            id: channel_id,
            name: name.clone(),
            owner: "plugin".to_string(),
        });
        name_to_id.insert(name, channel_id);
        
        Ok(channel_id)
    }
    
    pub async fn get_channel_by_name(&self, name: &str) -> Option<ChannelInfo> {
        let name_to_id = self.name_to_id.read().await;
        let channels = self.channels.read().await;
        
        name_to_id.get(name)
            .and_then(|id| channels.get(id))
            .cloned()
    }
    
    pub async fn get_channel_by_id(&self, id: u16) -> Option<ChannelInfo> {
        let channels = self.channels.read().await;
        channels.get(&id).cloned()
    }
    
    pub async fn list_channels(&self) -> Vec<ChannelInfo> {
        let channels = self.channels.read().await;
        channels.values().cloned().collect()
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}