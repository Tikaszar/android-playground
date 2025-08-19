use crate::components::*;
use crate::channel_manager::ChannelManager;
use crate::message_system::MessageSystem;
use anyhow::Result;
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Central UI state management for the conversational IDE
pub struct UiState {
    pub channel_manager: Arc<RwLock<ChannelManager>>,
    pub message_system: Arc<MessageSystem>,
    pub active_channel: Option<Uuid>,
    pub active_agents: Vec<AgentComponent>,
    pub task_queue: TaskQueueComponent,
}

impl UiState {
    pub fn new() -> Self {
        let channel_manager = Arc::new(RwLock::new(ChannelManager::new()));
        let message_system = Arc::new(MessageSystem::new(channel_manager.clone()));
        
        Self {
            channel_manager,
            message_system,
            active_channel: None,
            active_agents: Vec::new(),
            task_queue: TaskQueueComponent::new(),
        }
    }

    pub fn with_persistence(path: std::path::PathBuf) -> Self {
        let channel_manager = Arc::new(RwLock::new(ChannelManager::with_persistence(path)));
        let message_system = Arc::new(MessageSystem::new(channel_manager.clone()));
        
        Self {
            channel_manager,
            message_system,
            active_channel: None,
            active_agents: Vec::new(),
            task_queue: TaskQueueComponent::new(),
        }
    }

    // ========================================================================
    // Channel Operations
    // ========================================================================

    pub async fn create_system_channel(&mut self) -> Result<Uuid> {
        let channel_id = self
            .channel_manager
            .write()
            .await
            .create_channel(
                "System".to_string(),
                ChannelType::System,
                vec![], // System channel has no specific participants
            )
            .await?;
        
        // Send initial system message
        self.message_system
            .send_system_notification(channel_id, "Welcome to the Conversational IDE!".to_string())
            .await?;
        
        Ok(channel_id)
    }

    pub async fn create_general_channel(&mut self) -> Result<Uuid> {
        // Get all agent IDs
        let participants: Vec<AgentId> = self
            .channel_manager
            .read()
            .await
            .list_agents()
            .into_iter()
            .map(|a| a.id)
            .collect();
        
        let channel_id = self
            .channel_manager
            .write()
            .await
            .create_channel(
                "#general".to_string(),
                ChannelType::Group,
                participants,
            )
            .await?;
        
        // Send welcome message
        self.message_system
            .send_system_notification(
                channel_id,
                "This is the general discussion channel for all agents.".to_string(),
            )
            .await?;
        
        Ok(channel_id)
    }

    pub async fn create_dm_channel(
        &mut self,
        agent1: AgentId,
        agent2: AgentId,
    ) -> Result<Uuid> {
        let agent1_name = self
            .channel_manager
            .read()
            .await
            .get_agent(&agent1)
            .map(|a| a.name)
            .unwrap_or_else(|| "Unknown".to_string());
        
        let agent2_name = self
            .channel_manager
            .read()
            .await
            .get_agent(&agent2)
            .map(|a| a.name)
            .unwrap_or_else(|| "Unknown".to_string());
        
        let channel_name = format!("DM: {} ↔ {}", agent1_name, agent2_name);
        
        self.channel_manager
            .write()
            .await
            .create_channel(channel_name, ChannelType::Direct, vec![agent1, agent2])
            .await
    }

    pub fn set_active_channel(&mut self, channel_id: Uuid) {
        self.active_channel = Some(channel_id);
    }

    // ========================================================================
    // Agent Operations
    // ========================================================================

    pub async fn register_human_agent(&mut self, name: String) -> Result<AgentId> {
        let agent = AgentComponent {
            id: AgentId::new(),
            name,
            agent_type: AgentType::Human,
            status: AgentStatus::Idle,
            worktree_path: None,
            permissions: AgentPermissions {
                can_execute_commands: true,
                can_modify_files: true,
                can_create_worktrees: true,
                can_assign_tasks: true,
            },
            current_task: None,
            last_active: chrono::Utc::now(),
        };
        
        let agent_id = agent.id;
        self.channel_manager.write().await.register_agent(agent).await?;
        Ok(agent_id)
    }

    pub async fn register_orchestrator_agent(&mut self, name: String) -> Result<AgentId> {
        let agent = AgentComponent {
            id: AgentId::new(),
            name,
            agent_type: AgentType::Orchestrator,
            status: AgentStatus::Idle,
            worktree_path: Some(std::env::current_dir()?),
            permissions: AgentPermissions {
                can_execute_commands: true,
                can_modify_files: true,
                can_create_worktrees: true,
                can_assign_tasks: true,
            },
            current_task: None,
            last_active: chrono::Utc::now(),
        };
        
        let agent_id = agent.id;
        self.channel_manager.write().await.register_agent(agent).await?;
        Ok(agent_id)
    }

    pub async fn register_worker_agent(&mut self, name: String, worktree: Option<std::path::PathBuf>) -> Result<AgentId> {
        let agent = AgentComponent {
            id: AgentId::new(),
            name,
            agent_type: AgentType::Worker,
            status: AgentStatus::Idle,
            worktree_path: worktree,
            permissions: AgentPermissions {
                can_execute_commands: true,
                can_modify_files: true,
                can_create_worktrees: false,
                can_assign_tasks: false,
            },
            current_task: None,
            last_active: chrono::Utc::now(),
        };
        
        let agent_id = agent.id;
        self.channel_manager.write().await.register_agent(agent).await?;
        Ok(agent_id)
    }

    // ========================================================================
    // Task Operations
    // ========================================================================

    pub fn add_task(&mut self, task: Task) {
        self.task_queue.add_task(task);
    }

    pub async fn assign_next_task(&mut self) -> Result<Option<(AgentId, Task)>> {
        // Find an idle worker
        let idle_agents = self
            .channel_manager
            .read()
            .await
            .list_agents()
            .into_iter()
            .filter(|a| {
                matches!(a.agent_type, AgentType::Worker)
                    && matches!(a.status, AgentStatus::Idle)
            })
            .collect::<Vec<_>>();
        
        if let Some(agent) = idle_agents.first() {
            if let Some(task) = self.task_queue.assign_task(agent.id) {
                // Update agent status
                self.channel_manager
                    .write()
                    .await
                    .update_agent_status(agent.id, AgentStatus::Busy)
                    .await?;
                
                return Ok(Some((agent.id, task)));
            }
        }
        
        Ok(None)
    }

    pub async fn complete_task(
        &mut self,
        agent_id: AgentId,
        result: TaskResult,
    ) -> Result<()> {
        self.task_queue.complete_task(agent_id, result);
        
        // Update agent status back to idle
        self.channel_manager
            .write()
            .await
            .update_agent_status(agent_id, AgentStatus::Idle)
            .await?;
        
        Ok(())
    }

    // ========================================================================
    // Message Handling
    // ========================================================================

    pub async fn handle_chat_message(&mut self, msg: serde_json::Value) -> Result<()> {
        // Extract message fields
        let channel_id = msg.get("channel_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        let content = msg.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let author_id = msg.get("author_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(AgentId)
            .unwrap_or(AgentId(Uuid::nil()));
        
        // Send message to the appropriate channel
        if let Some(channel_id) = channel_id {
            self.message_system
                .send_text_message(channel_id, author_id, content.to_string())
                .await?;
            
            tracing::debug!("Handled chat message in channel {:?}", channel_id);
        } else if let Some(channel_id) = self.active_channel {
            // If no channel specified, use active channel
            self.message_system
                .send_text_message(channel_id, author_id, content.to_string())
                .await?;
            
            tracing::debug!("Handled chat message in active channel {:?}", channel_id);
        } else {
            tracing::warn!("No channel specified for chat message and no active channel");
        }
        
        Ok(())
    }

    // ========================================================================
    // Persistence Operations
    // ========================================================================

    pub async fn save_state(&self) -> Result<()> {
        // Save current UI state to disk
        let persistence_path = std::path::PathBuf::from("/data/data/com.termux/files/home/.android-playground/conversations");
        
        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(&persistence_path).await?;
        
        // Save channels
        let channels = self.channel_manager.read().await.list_channels();
        let channels_json = serde_json::to_string_pretty(&channels)?;
        let channels_path = persistence_path.join("channels.json");
        tokio::fs::write(channels_path, channels_json).await?;
        
        // Save agents
        let agents = self.channel_manager.read().await.list_agents();
        let agents_json = serde_json::to_string_pretty(&agents)?;
        let agents_path = persistence_path.join("agents.json");
        tokio::fs::write(agents_path, agents_json).await?;
        
        // Save task queue
        let tasks_json = serde_json::to_string_pretty(&self.task_queue)?;
        let tasks_path = persistence_path.join("task_queue.json");
        tokio::fs::write(tasks_path, tasks_json).await?;
        
        // Save active channel
        if let Some(active_channel) = self.active_channel {
            let state_json = serde_json::json!({
                "active_channel": active_channel,
                "timestamp": chrono::Utc::now()
            });
            let state_path = persistence_path.join("ui_state.json");
            tokio::fs::write(state_path, serde_json::to_string_pretty(&state_json)?).await?;
        }
        
        tracing::info!("UI state saved to disk");
        Ok(())
    }

    // ========================================================================
    // Initialize Default Setup
    // ========================================================================

    pub async fn initialize_default_setup(&mut self) -> Result<()> {
        // Create default agents
        let human = self.register_human_agent("Human".to_string()).await?;
        let orchestrator = self.register_orchestrator_agent("Orchestrator".to_string()).await?;
        
        // Create default channels
        let system_channel = self.create_system_channel().await?;
        let general_channel = self.create_general_channel().await?;
        
        // Set general as active channel
        self.set_active_channel(general_channel);
        
        // Send welcome message
        self.message_system
            .send_text_message(
                general_channel,
                orchestrator,
                "Hello! I'm the Orchestrator. I'll help manage tasks and coordinate work between agents.".to_string(),
            )
            .await?;
        
        tracing::info!("Default UI setup initialized");
        Ok(())
    }
}