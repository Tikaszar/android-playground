use crate::components::*;
use crate::channel_manager::ChannelManager;
use crate::message_system::MessageSystem;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

/// Central UI state management for the conversational IDE
pub struct UiState {
    pub channel_manager: Arc<ChannelManager>,
    pub message_system: Arc<MessageSystem>,
    pub active_channel: Option<Uuid>,
    pub active_agents: Vec<AgentComponent>,
    pub task_queue: TaskQueueComponent,
}

impl UiState {
    pub fn new() -> Self {
        let channel_manager = Arc::new(ChannelManager::new());
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
        let channel_manager = Arc::new(ChannelManager::with_persistence(path));
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
            .list_agents()
            .into_iter()
            .map(|a| a.id)
            .collect();
        
        let channel_id = self
            .channel_manager
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
            .get_agent(&agent1)
            .map(|a| a.name)
            .unwrap_or_else(|| "Unknown".to_string());
        
        let agent2_name = self
            .channel_manager
            .get_agent(&agent2)
            .map(|a| a.name)
            .unwrap_or_else(|| "Unknown".to_string());
        
        let channel_name = format!("DM: {} ↔ {}", agent1_name, agent2_name);
        
        self.channel_manager
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
        self.channel_manager.register_agent(agent).await?;
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
        self.channel_manager.register_agent(agent).await?;
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
        self.channel_manager.register_agent(agent).await?;
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
            .update_agent_status(agent_id, AgentStatus::Idle)
            .await?;
        
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