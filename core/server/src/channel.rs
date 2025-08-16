use dashmap::DashMap;
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
    channels: DashMap<u16, ChannelInfo>,
    name_to_id: DashMap<String, u16>,
    next_plugin_channel: Arc<RwLock<u16>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        let manager = Self {
            channels: DashMap::new(),
            name_to_id: DashMap::new(),
            next_plugin_channel: Arc::new(RwLock::new(1000)),
        };
        
        manager.channels.insert(0, ChannelInfo {
            id: 0,
            name: "control".to_string(),
            owner: "core".to_string(),
        });
        manager.name_to_id.insert("control".to_string(), 0);
        
        manager
    }
    
    pub fn register_system(&self, name: String, channel_id: u16) -> Result<u16> {
        if channel_id == 0 || channel_id >= 1000 {
            bail!("System channels must be in range 1-999");
        }
        
        if self.channels.contains_key(&channel_id) {
            bail!("Channel {} already registered", channel_id);
        }
        
        if self.name_to_id.contains_key(&name) {
            bail!("Channel name '{}' already registered", name);
        }
        
        self.channels.insert(channel_id, ChannelInfo {
            id: channel_id,
            name: name.clone(),
            owner: "system".to_string(),
        });
        self.name_to_id.insert(name, channel_id);
        
        Ok(channel_id)
    }
    
    pub async fn register_plugin(&self, name: String) -> Result<u16> {
        if self.name_to_id.contains_key(&name) {
            bail!("Channel name '{}' already registered", name);
        }
        
        let mut next_id = self.next_plugin_channel.write().await;
        let channel_id = *next_id;
        *next_id += 1;
        
        self.channels.insert(channel_id, ChannelInfo {
            id: channel_id,
            name: name.clone(),
            owner: "plugin".to_string(),
        });
        self.name_to_id.insert(name, channel_id);
        
        Ok(channel_id)
    }
    
    pub fn get_channel_by_name(&self, name: &str) -> Option<ChannelInfo> {
        self.name_to_id.get(name)
            .and_then(|id| self.channels.get(&*id))
            .map(|entry| entry.clone())
    }
    
    pub fn get_channel_by_id(&self, id: u16) -> Option<ChannelInfo> {
        self.channels.get(&id).map(|entry| entry.clone())
    }
    
    pub fn list_channels(&self) -> Vec<ChannelInfo> {
        self.channels.iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}