use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub id: u16,
    pub name: String,
    pub owner: String,
}

pub struct ChannelManager {
    channels: HashMap<u16, ChannelInfo>,
    name_to_id: HashMap<String, u16>,
}

impl ChannelManager {
    pub fn new() -> Self {
        let mut manager = Self {
            channels: HashMap::new(),
            name_to_id: HashMap::new(),
        };
        
        manager.channels.insert(0, ChannelInfo {
            id: 0,
            name: "control".to_string(),
            owner: "core".to_string(),
        });
        manager.name_to_id.insert("control".to_string(), 0);
        
        manager
    }
    
    pub fn register_system(&mut self, name: String, channel_id: u16) -> Result<u16, String> {
        if channel_id == 0 || channel_id >= 1000 {
            return Err("System channels must be in range 1-999".to_string());
        }
        
        if self.channels.contains_key(&channel_id) {
            return Err(format!("Channel {} already registered", channel_id));
        }
        
        if self.name_to_id.contains_key(&name) {
            return Err(format!("Channel name '{}' already registered", name));
        }
        
        self.channels.insert(channel_id, ChannelInfo {
            id: channel_id,
            name: name.clone(),
            owner: "system".to_string(),
        });
        self.name_to_id.insert(name, channel_id);
        
        Ok(channel_id)
    }
    
    pub fn register_plugin(&mut self, name: String, channel_id: u16) -> Result<u16, String> {
        if self.name_to_id.contains_key(&name) {
            return Err(format!("Channel name '{}' already registered", name));
        }
        
        self.channels.insert(channel_id, ChannelInfo {
            id: channel_id,
            name: name.clone(),
            owner: "plugin".to_string(),
        });
        self.name_to_id.insert(name, channel_id);
        
        Ok(channel_id)
    }
    
    pub fn get_channel_by_name(&self, name: &str) -> Option<&ChannelInfo> {
        self.name_to_id.get(name)
            .and_then(|id| self.channels.get(id))
    }
    
    pub fn get_channel_by_id(&self, id: u16) -> Option<&ChannelInfo> {
        self.channels.get(&id)
    }
}