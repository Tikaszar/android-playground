use crate::components::*;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Manages all chat channels and their participants
pub struct ChannelManager {
    channels: Arc<DashMap<Uuid, ChannelComponent>>,
    messages: Arc<DashMap<Uuid, Arc<RwLock<Vec<MessageComponent>>>>>,
    agents: Arc<DashMap<AgentId, AgentComponent>>,
    persistence_path: Option<PathBuf>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            messages: Arc::new(DashMap::new()),
            agents: Arc::new(DashMap::new()),
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
        &self,
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
        self.messages
            .insert(channel_id, Arc::new(RwLock::new(Vec::new())));

        // Persist if enabled
        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        tracing::info!("Created channel '{}' with ID {}", name, channel_id);
        Ok(channel_id)
    }

    pub async fn delete_channel(&self, channel_id: Uuid) -> Result<()> {
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
        self.channels.iter().map(|c| c.clone()).collect()
    }

    pub fn list_channels_for_agent(&self, agent_id: &AgentId) -> Vec<ChannelComponent> {
        self.channels
            .iter()
            .filter(|c| c.participants.contains(agent_id))
            .map(|c| c.clone())
            .collect()
    }

    pub async fn add_participant(&self, channel_id: Uuid, agent_id: AgentId) -> Result<()> {
        let mut channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or_else(|| anyhow!("Channel {} not found", channel_id))?;

        if !channel.participants.contains(&agent_id) {
            channel.participants.push(agent_id);
        }

        drop(channel); // Release the lock

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        Ok(())
    }

    pub async fn remove_participant(&self, channel_id: Uuid, agent_id: AgentId) -> Result<()> {
        let mut channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or_else(|| anyhow!("Channel {} not found", channel_id))?;

        channel.participants.retain(|id| id != &agent_id);

        drop(channel); // Release the lock

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        Ok(())
    }

    // ========================================================================
    // Message Management
    // ========================================================================

    pub async fn send_message(
        &self,
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
        drop(channel);

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
        if let Some(messages) = self.messages.get(&channel_id) {
            let mut messages = messages.write().await;
            messages.push(message);
        }

        // Update channel's last message time
        if let Some(mut channel) = self.channels.get_mut(&channel_id) {
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
            .map(|messages| {
                let messages = messages.blocking_read();
                messages.clone()
            })
    }

    pub async fn get_messages_paginated(
        &self,
        channel_id: Uuid,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<MessageComponent>> {
        if let Some(messages) = self.messages.get(&channel_id) {
            let messages = messages.read().await;
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
        &self,
        channel_id: Uuid,
        message_id: Uuid,
        new_state: BubbleState,
    ) -> Result<()> {
        if let Some(messages) = self.messages.get(&channel_id) {
            let mut messages = messages.write().await;
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

    pub async fn toggle_bubble_state(&self, channel_id: Uuid, message_id: Uuid) -> Result<()> {
        if let Some(messages) = self.messages.get(&channel_id) {
            let mut messages = messages.write().await;
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

    pub async fn register_agent(&self, agent: AgentComponent) -> Result<()> {
        let agent_id = agent.id;
        self.agents.insert(agent_id, agent);

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        tracing::info!("Registered agent {:?}", agent_id);
        Ok(())
    }

    pub async fn update_agent_status(&self, agent_id: AgentId, status: AgentStatus) -> Result<()> {
        let mut agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow!("Agent {:?} not found", agent_id))?;
        
        agent.status = status;
        agent.last_active = chrono::Utc::now();
        
        drop(agent);

        if self.persistence_path.is_some() {
            self.save_to_disk().await?;
        }

        Ok(())
    }

    pub fn get_agent(&self, agent_id: &AgentId) -> Option<AgentComponent> {
        self.agents.get(agent_id).map(|a| a.clone())
    }

    pub fn list_agents(&self) -> Vec<AgentComponent> {
        self.agents.iter().map(|a| a.clone()).collect()
    }

    pub fn list_online_agents(&self) -> Vec<AgentComponent> {
        self.agents
            .iter()
            .filter(|a| !matches!(a.status, AgentStatus::Offline))
            .map(|a| a.clone())
            .collect()
    }

    // ========================================================================
    // Persistence
    // ========================================================================

    pub async fn save_to_disk(&self) -> Result<()> {
        if let Some(path) = &self.persistence_path {
            #[derive(Serialize, Deserialize)]
            struct PersistedState {
                channels: Vec<ChannelComponent>,
                messages: HashMap<Uuid, Vec<MessageComponent>>,
                agents: Vec<AgentComponent>,
            }
            
            let mut messages_map = HashMap::new();
            for entry in self.messages.iter() {
                let channel_id = *entry.key();
                let messages = entry.value().read().await;
                messages_map.insert(channel_id, messages.clone());
            }

            let state = PersistedState {
                channels: self.list_channels(),
                messages: messages_map,
                agents: self.list_agents(),
            };

            let json = serde_json::to_string_pretty(&state)?;
            tokio::fs::write(path, json).await?;
            
            tracing::debug!("Saved channel state to {:?}", path);
        }
        Ok(())
    }

    pub async fn load_from_disk(&self) -> Result<()> {
        if let Some(path) = &self.persistence_path {
            if path.exists() {
                #[derive(Serialize, Deserialize)]
                struct PersistedState {
                    channels: Vec<ChannelComponent>,
                    messages: HashMap<Uuid, Vec<MessageComponent>>,
                    agents: Vec<AgentComponent>,
                }
                
                let json = tokio::fs::read_to_string(path).await?;
                let state: PersistedState = serde_json::from_str(&json)?;

                // Clear existing data
                self.channels.clear();
                self.messages.clear();
                self.agents.clear();

                // Load channels
                for channel in state.channels {
                    self.channels.insert(channel.id, channel);
                }

                // Load messages
                for (channel_id, messages) in state.messages {
                    self.messages
                        .insert(channel_id, Arc::new(RwLock::new(messages)));
                }

                // Load agents
                for agent in state.agents {
                    self.agents.insert(agent.id, agent);
                }

                tracing::info!("Loaded channel state from {:?}", path);
            }
        }
        Ok(())
    }

    // ========================================================================
    // Search and Filtering
    // ========================================================================

    pub async fn search_messages(&self, query: &str) -> Vec<(ChannelComponent, MessageComponent)> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for channel_entry in self.channels.iter() {
            let channel = channel_entry.clone();
            if let Some(messages) = self.messages.get(&channel.id) {
                let messages = messages.read().await;
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
}