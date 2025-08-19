use crate::components::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Manages all chat channels and their participants
pub struct ChannelManager {
    channels: HashMap<Uuid, ChannelComponent>,
    messages: HashMap<Uuid, Vec<MessageComponent>>,
    agents: HashMap<AgentId, AgentComponent>,
    persistence_path: Option<PathBuf>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            messages: HashMap::new(),
            agents: HashMap::new(),
            persistence_path: None,
        }
    }

    pub fn with_persistence(path: PathBuf) -> Self {
        let mut manager = Self::new();
        manager.persistence_path = Some(path);
        manager
    }

    // ========================================================================
    // Channel Management
    // ========================================================================

    pub async fn create_channel(
        &mut self,
        name: String,
        channel_type: ChannelType,
        participants: Vec<AgentId>,
    ) -> Result<Uuid> {
        // Validate participants exist
        for participant in &participants {
            if !self.agents.contains_key(participant) {
                return Err(anyhow!("Agent {:?} does not exist", participant));
            }
        }

        let channel = ChannelComponent {
            id: Uuid::new_v4(),
            name: name.clone(),
            channel_type,
            participants,
            created_at: chrono::Utc::now(),
            last_message_at: None,
            unread_count: 0,
        };

        let channel_id = channel.id;
        self.channels.insert(channel_id, channel);
        self.messages.insert(channel_id, Vec::new());

        // Persist if enabled
        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        tracing::info!("Created channel '{}' with ID {}", name, channel_id);
        Ok(channel_id)
    }

    pub async fn delete_channel(&mut self, channel_id: Uuid) -> Result<()> {
        self.channels
            .remove(&channel_id)
            .ok_or_else(|| anyhow!("Channel {} not found", channel_id))?;
        self.messages.remove(&channel_id);

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        tracing::info!("Deleted channel {}", channel_id);
        Ok(())
    }

    pub fn get_channel(&self, channel_id: &Uuid) -> Option<ChannelComponent> {
        self.channels.get(channel_id).map(|c| c.clone())
    }

    pub fn list_channels(&self) -> Vec<ChannelComponent> {
        self.channels.values().cloned().collect()
    }

    pub fn list_channels_for_agent(&self, agent_id: &AgentId) -> Vec<ChannelComponent> {
        self.channels
            .values()
            .filter(|c| c.participants.contains(agent_id))
            .cloned()
            .collect()
    }

    pub async fn add_participant(&mut self, channel_id: Uuid, agent_id: AgentId) -> Result<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or_else(|| anyhow!("Channel {} not found", channel_id))?;

        if !channel.participants.contains(&agent_id) {
            channel.participants.push(agent_id);
        }

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        Ok(())
    }

    pub async fn remove_participant(&mut self, channel_id: Uuid, agent_id: AgentId) -> Result<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or_else(|| anyhow!("Channel {} not found", channel_id))?;

        channel.participants.retain(|id| id != &agent_id);

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        Ok(())
    }

    // ========================================================================
    // Message Management
    // ========================================================================

    pub async fn send_message(
        &mut self,
        channel_id: Uuid,
        sender: AgentId,
        content: MessageContent,
    ) -> Result<Uuid> {
        // Verify channel exists
        if !self.channels.contains_key(&channel_id) {
            return Err(anyhow!("Channel {} not found", channel_id));
        }

        // Verify sender is participant
        let channel = self.channels.get(&channel_id).unwrap();
        if !channel.participants.contains(&sender) {
            return Err(anyhow!("Agent {:?} is not a participant in this channel", sender));
        }

        let message = MessageComponent {
            id: Uuid::new_v4(),
            channel_id,
            sender,
            content,
            timestamp: chrono::Utc::now(),
            bubble_state: BubbleState::Expanded,
            edited_at: None,
            reply_to: None,
        };

        let message_id = message.id;

        // Add message to channel
        if let Some(messages) = self.messages.get_mut(&channel_id) {
            messages.push(message);
        }

        // Update channel's last message time
        if let Some(channel) = self.channels.get_mut(&channel_id) {
            channel.last_message_at = Some(chrono::Utc::now());
        }

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        Ok(message_id)
    }

    pub async fn get_messages(&self, channel_id: Uuid) -> Result<Vec<MessageComponent>> {
        self.messages
            .get(&channel_id)
            .ok_or_else(|| anyhow!("Channel {} not found", channel_id))
            .map(|messages| messages.clone())
    }

    pub async fn get_messages_paginated(
        &self,
        channel_id: Uuid,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<MessageComponent>> {
        if let Some(messages) = self.messages.get(&channel_id) {
            let total = messages.len();
            
            if offset >= total {
                return Ok(Vec::new());
            }

            let end = (offset + limit).min(total);
            Ok(messages[offset..end].to_vec())
        } else {
            Err(anyhow!("Channel {} not found", channel_id))
        }
    }

    pub async fn update_bubble_state(
        &mut self,
        channel_id: Uuid,
        message_id: Uuid,
        new_state: BubbleState,
    ) -> Result<()> {
        if let Some(messages) = self.messages.get_mut(&channel_id) {
            if let Some(message) = messages.iter_mut().find(|m| m.id == message_id) {
                message.bubble_state = new_state;
                
                if self.persistence_path.is_some() {
                    self.save_to_disk().await?;
                }
                Ok(())
            } else {
                Err(anyhow!("Message {} not found", message_id))
            }
        } else {
            Err(anyhow!("Channel {} not found", channel_id))
        }
    }

    pub async fn toggle_bubble_state(&mut self, channel_id: Uuid, message_id: Uuid) -> Result<()> {
        if let Some(messages) = self.messages.get_mut(&channel_id) {
            if let Some(message) = messages.iter_mut().find(|m| m.id == message_id) {
                message.bubble_state = match message.bubble_state {
                    BubbleState::Collapsed => BubbleState::Compressed,
                    BubbleState::Compressed => BubbleState::Expanded,
                    BubbleState::Expanded => BubbleState::Collapsed,
                };
                
                if self.persistence_path.is_some() {
                    self.save_to_disk().await?;
                }
                Ok(())
            } else {
                Err(anyhow!("Message {} not found", message_id))
            }
        } else {
            Err(anyhow!("Channel {} not found", channel_id))
        }
    }

    // ========================================================================
    // Agent Management
    // ========================================================================

    pub async fn register_agent(&mut self, agent: AgentComponent) -> Result<()> {
        let agent_id = agent.id;
        self.agents.insert(agent_id, agent);

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        tracing::info!("Registered agent {:?}", agent_id);
        Ok(())
    }

    pub async fn update_agent_status(&mut self, agent_id: AgentId, status: AgentStatus) -> Result<()> {
        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow!("Agent {:?} not found", agent_id))?;
        
        agent.status = status;
        agent.last_active = chrono::Utc::now();

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        Ok(())
    }

    pub fn get_agent(&self, agent_id: &AgentId) -> Option<AgentComponent> {
        self.agents.get(agent_id).cloned()
    }

    pub fn list_agents(&self) -> Vec<AgentComponent> {
        self.agents.values().cloned().collect()
    }

    pub fn list_online_agents(&self) -> Vec<AgentComponent> {
        self.agents
            .values()
            .filter(|a| !matches!(a.status, AgentStatus::Offline))
            .cloned()
            .collect()
    }


    // ========================================================================
    // Search and Filtering
    // ========================================================================

    pub async fn search_messages(&self, query: &str) -> Vec<(ChannelComponent, MessageComponent)> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for (channel_id, channel) in self.channels.iter() {
            if let Some(messages) = self.messages.get(channel_id) {
                for message in messages.iter() {
                    let matches = match &message.content {
                        MessageContent::Text(text) => text.to_lowercase().contains(&query_lower),
                        MessageContent::Code { content, .. } => {
                            content.to_lowercase().contains(&query_lower)
                        }
                        MessageContent::SystemNotification(text) => {
                            text.to_lowercase().contains(&query_lower)
                        }
                        _ => false,
                    };

                    if matches {
                        results.push((channel.clone(), message.clone()));
                    }
                }
            }
        }

        results
    }
    
    // ========================================================================
    // Persistence  
    // ========================================================================
    
    pub async fn save_to_disk(&self) -> Result<()> {
        let path = match &self.persistence_path {
            Some(p) => p,
            None => return Ok(()), // No persistence configured
        };
        
        // Create persistence directory if it doesn't exist
        tokio::fs::create_dir_all(path).await?;
        
        // Save channels
        let channels_path = path.join("channels.json");
        let channels: Vec<ChannelComponent> = self.channels
            .values()
            .cloned()
            .collect();
        let channels_json = serde_json::to_string_pretty(&channels)?;
        tokio::fs::write(channels_path, channels_json).await?;
        
        // Save agents
        let agents_path = path.join("agents.json");
        let agents: Vec<AgentComponent> = self.agents
            .values()
            .cloned()
            .collect();
        let agents_json = serde_json::to_string_pretty(&agents)?;
        tokio::fs::write(agents_path, agents_json).await?;
        
        // Save messages for each channel
        for (channel_id, messages) in self.messages.iter() {
            let messages_path = path.join(format!("messages_{}.json", channel_id));
            let messages_json = serde_json::to_string_pretty(messages)?;
            tokio::fs::write(messages_path, messages_json).await?;
        }
        
        tracing::info!("Saved conversations to disk at {:?}", path);
        Ok(())
    }
    
    pub async fn load_from_disk(&mut self) -> Result<()> {
        let path = match &self.persistence_path {
            Some(p) => p,
            None => return Ok(()), // No persistence configured
        };
        
        if !path.exists() {
            return Ok(()); // Nothing to load
        }
        
        // Load channels
        let channels_path = path.join("channels.json");
        if channels_path.exists() {
            let channels_json = tokio::fs::read_to_string(channels_path).await?;
            let channels: Vec<ChannelComponent> = serde_json::from_str(&channels_json)?;
            
            for channel in channels {
                self.channels.insert(channel.id, channel);
            }
        }
        
        // Load agents
        let agents_path = path.join("agents.json");
        if agents_path.exists() {
            let agents_json = tokio::fs::read_to_string(agents_path).await?;
            let agents: Vec<AgentComponent> = serde_json::from_str(&agents_json)?;
            
            for agent in agents {
                self.agents.insert(agent.id, agent);
            }
        }
        
        // Load messages for each channel
        for (channel_id, _channel) in self.channels.iter() {
            let messages_path = path.join(format!("messages_{}.json", channel_id));
            
            if messages_path.exists() {
                let messages_json = tokio::fs::read_to_string(messages_path).await?;
                let messages: Vec<MessageComponent> = serde_json::from_str(&messages_json)?;
                
                self.messages.insert(*channel_id, messages);
            }
        }
        
        tracing::info!("Loaded conversations from disk at {:?}", path);
        Ok(())
    }
    
    pub async fn auto_save(&self) -> Result<()> {
        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }
        Ok(())
    }
}