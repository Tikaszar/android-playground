use crate::components::*;
use crate::channel_manager::ChannelManager;
use crate::message_system::MessageSystem;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, debug, warn};

/// Manages agent coordination and task assignment
pub struct Orchestrator {
    channel_manager: Arc<RwLock<ChannelManager>>,
    message_system: Arc<MessageSystem>,
    task_queue: Arc<RwLock<TaskQueueComponent>>,
    orchestrator_id: AgentId,
    system_channel: Option<Uuid>,
}

impl Orchestrator {
    pub fn new(
        channel_manager: Arc<RwLock<ChannelManager>>,
        message_system: Arc<MessageSystem>,
    ) -> Self {
        Self {
            channel_manager,
            message_system,
            task_queue: Arc::new(RwLock::new(TaskQueueComponent::new())),
            orchestrator_id: AgentId(Uuid::nil()), // System agent
            system_channel: None,
        }
    }

    // ========================================================================
    // Initialization
    // ========================================================================

    pub async fn initialize(&mut self) -> Result<()> {
        // Register orchestrator agent
        let orchestrator = AgentComponent {
            id: AgentId::new(),
            name: "Orchestrator".to_string(),
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

        self.orchestrator_id = orchestrator.id;
        self.channel_manager.write().await.register_agent(orchestrator).await?;

        // Create system channel for notifications
        self.system_channel = Some(
            self.channel_manager
                .write()
                .await
                .create_channel(
                    "System".to_string(),
                    ChannelType::System,
                    vec![self.orchestrator_id],
                )
                .await?
        );

        info!("Orchestrator initialized with ID: {:?}", self.orchestrator_id);
        Ok(())
    }

    // ========================================================================
    // Task Management
    // ========================================================================

    pub async fn create_task(
        &self,
        title: String,
        description: String,
        priority: TaskPriority,
        context_files: Vec<PathBuf>,
    ) -> Result<TaskId> {
        let task = Task {
            id: TaskId::new(),
            title: title.clone(),
            description,
            priority,
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now(),
            assigned_at: None,
            completed_at: None,
            dependencies: Vec::new(),
            context_files,
        };

        let task_id = task.id;
        
        // Add to queue
        let mut queue = self.task_queue.write().await;
        queue.add_task(task);

        // Send notification
        if let Some(channel) = self.system_channel {
            self.message_system
                .send_system_notification(
                    channel,
                    format!("📋 New task created: {}", title),
                )
                .await?;
        }

        info!("Created task {:?}: {}", task_id, title);
        Ok(task_id)
    }

    pub async fn assign_next_task(&self) -> Result<Option<(AgentId, Task)>> {
        // Find idle workers
        let agents = self.channel_manager.read().await.list_agents();
        let idle_workers: Vec<_> = agents
            .into_iter()
            .filter(|a| {
                matches!(a.agent_type, AgentType::Worker)
                    && matches!(a.status, AgentStatus::Idle)
            })
            .collect();

        if idle_workers.is_empty() {
            debug!("No idle workers available");
            return Ok(None);
        }

        // Assign task to first available worker
        let worker = &idle_workers[0];
        let mut queue = self.task_queue.write().await;
        
        if let Some(task) = queue.assign_task(worker.id) {
            // Update agent status
            self.channel_manager
                .write()
                .await
                .update_agent_status(worker.id, AgentStatus::Busy)
                .await?;

            // Send assignment notification
            if let Some(channel) = self.system_channel {
                self.message_system
                    .send_system_notification(
                        channel,
                        format!(
                            "✅ Task '{}' assigned to {}",
                            task.title, worker.name
                        ),
                    )
                    .await?;
            }

            // Send DM to worker with task details
            let dm_channel = self.channel_manager
                .write()
                .await
                .create_channel(
                    format!("Task: {}", task.title),
                    ChannelType::Direct,
                    vec![self.orchestrator_id, worker.id],
                )
                .await?;

            self.message_system
                .send_text_message(
                    dm_channel,
                    self.orchestrator_id,
                    format!(
                        "You've been assigned a new task:\n\n**{}**\n\n{}",
                        task.title, task.description
                    ),
                )
                .await?;

            info!("Assigned task {:?} to agent {:?}", task.id, worker.id);
            Ok(Some((worker.id, task)))
        } else {
            Ok(None)
        }
    }

    pub async fn complete_task(
        &self,
        agent_id: AgentId,
        result: TaskResult,
    ) -> Result<()> {
        let mut queue = self.task_queue.write().await;
        queue.complete_task(agent_id, result.clone());

        // Update agent status
        self.channel_manager
            .write()
            .await
            .update_agent_status(agent_id, AgentStatus::Idle)
            .await?;

        // Send completion notification
        if let Some(channel) = self.system_channel {
            let status_icon = if result.success { "✅" } else { "❌" };
            self.message_system
                .send_system_notification(
                    channel,
                    format!(
                        "{} Task {:?} completed: {}",
                        status_icon, result.task_id, result.output
                    ),
                )
                .await?;
        }

        info!("Task {:?} completed by agent {:?}", result.task_id, agent_id);
        Ok(())
    }

    // ========================================================================
    // Worker Management
    // ========================================================================

    pub async fn create_worker(
        &self,
        name: String,
        worktree_path: PathBuf,
    ) -> Result<AgentId> {
        // Create git worktree if it doesn't exist
        if !worktree_path.exists() {
            use tokio::process::Command;
            
            let branch_name = format!("worker-{}", name.to_lowercase().replace(' ', "-"));
            let output = Command::new("git")
                .args(&["worktree", "add", worktree_path.to_str().unwrap(), "-b", &branch_name])
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("Failed to create worktree: {}", stderr));
            }

            info!("Created git worktree at {:?}", worktree_path);
        }

        // Register worker agent
        let worker = AgentComponent {
            id: AgentId::new(),
            name: name.clone(),
            agent_type: AgentType::Worker,
            status: AgentStatus::Idle,
            worktree_path: Some(worktree_path.clone()),
            permissions: AgentPermissions {
                can_execute_commands: true,
                can_modify_files: true,
                can_create_worktrees: false,
                can_assign_tasks: false,
            },
            current_task: None,
            last_active: chrono::Utc::now(),
        };

        let worker_id = worker.id;
        self.channel_manager.write().await.register_agent(worker).await?;

        // Add worker to general channel
        let channels = self.channel_manager.read().await.list_channels();
        if let Some(general) = channels.iter().find(|c| c.name == "#general") {
            self.channel_manager
                .write()
                .await
                .add_participant(general.id, worker_id)
                .await?;

            // Send welcome message
            self.message_system
                .send_text_message(
                    general.id,
                    worker_id,
                    format!("👋 Hello! I'm {}, ready to work on tasks.", name),
                )
                .await?;
        }

        info!("Created worker {:?}: {} at {:?}", worker_id, name, worktree_path);
        Ok(worker_id)
    }

    pub async fn remove_worker(&self, agent_id: AgentId) -> Result<()> {
        // Get worker info
        let agent = self.channel_manager
            .read()
            .await
            .get_agent(&agent_id)
            .ok_or_else(|| anyhow!("Agent not found"))?;

        if !matches!(agent.agent_type, AgentType::Worker) {
            return Err(anyhow!("Agent is not a worker"));
        }

        // Remove git worktree if it exists
        if let Some(worktree_path) = &agent.worktree_path {
            use tokio::process::Command;
            
            let output = Command::new("git")
                .args(&["worktree", "remove", worktree_path.to_str().unwrap()])
                .output()
                .await?;

            if !output.status.success() {
                warn!("Failed to remove worktree: {}", String::from_utf8_lossy(&output.stderr));
            } else {
                info!("Removed git worktree at {:?}", worktree_path);
            }
        }

        // Remove from all channels
        let channels = self.channel_manager.read().await.list_channels_for_agent(&agent_id);
        for channel in channels {
            self.channel_manager
                .write()
                .await
                .remove_participant(channel.id, agent_id)
                .await?;
        }

        info!("Removed worker {:?}: {}", agent_id, agent.name);
        Ok(())
    }

    // ========================================================================
    // Context Management
    // ========================================================================

    pub async fn update_context_files(&self, files: Vec<PathBuf>) -> Result<()> {
        // Write CONTEXT.md file
        let context_path = PathBuf::from("CONTEXT.md");
        let mut content = String::from("# Current Context\n\n");
        
        for file in &files {
            if file.exists() {
                content.push_str(&format!("## {}\n", file.display()));
                let file_content = tokio::fs::read_to_string(file).await?;
                content.push_str(&format!("```\n{}\n```\n\n", file_content));
            }
        }

        tokio::fs::write(context_path, content).await?;
        
        info!("Updated context with {} files", files.len());
        Ok(())
    }

    pub async fn switch_worker_context(
        &self,
        agent_id: AgentId,
        new_worktree: PathBuf,
    ) -> Result<()> {
        // Get agent
        let agent = self.channel_manager
            .read()
            .await
            .get_agent(&agent_id)
            .ok_or_else(|| anyhow!("Agent not found"))?;

        if !matches!(agent.agent_type, AgentType::Worker) {
            return Err(anyhow!("Agent is not a worker"));
        }

        // Execute context switch command
        use tokio::process::Command;
        
        let cmd_string = format!("cd {} && claude --continue", new_worktree.display());
        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd_string)
            .current_dir(&new_worktree)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Context switch failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Send notification
        if let Some(channel) = self.system_channel {
            self.message_system
                .send_system_notification(
                    channel,
                    format!(
                        "🔄 {} switched context to {:?}",
                        agent.name, new_worktree
                    ),
                )
                .await?;
        }

        info!("Switched context for {:?} to {:?}", agent_id, new_worktree);
        Ok(())
    }

    // ========================================================================
    // Update Processing
    // ========================================================================

    pub async fn process_pending_updates(&mut self, _delta_time: f32) -> Result<()> {
        // Process any pending task assignments
        if let Some((agent_id, task)) = self.assign_next_task().await? {
            tracing::info!("Auto-assigned task {:?} to agent {:?}", task.id, agent_id);
        }
        
        // Check for timed-out tasks
        let queue = self.task_queue.read().await;
        let now = chrono::Utc::now();
        
        for task in queue.get_in_progress_tasks() {
            if let Some(assigned_at) = task.assigned_at {
                let duration = now.signed_duration_since(assigned_at);
                if duration.num_hours() > 1 {
                    tracing::warn!("Task {:?} has been in progress for over 1 hour", task.id);
                    
                    // Send reminder notification
                    if let Some(channel) = self.system_channel {
                        self.message_system
                            .send_system_notification(
                                channel,
                                format!("⏰ Task '{}' has been running for {} minutes", 
                                    task.title, 
                                    duration.num_minutes()
                                ),
                            )
                            .await?;
                    }
                }
            }
        }
        
        // Process any pending messages from workers
        let agents = self.channel_manager.read().await.list_agents();
        for agent in agents {
            if matches!(agent.agent_type, AgentType::Worker) {
                // Check if worker needs attention
                let time_since_active = now.signed_duration_since(agent.last_active);
                if time_since_active.num_minutes() > 30 && matches!(agent.status, AgentStatus::Busy) {
                    tracing::debug!("Worker {:?} hasn't updated in {} minutes", 
                        agent.id, time_since_active.num_minutes());
                }
            }
        }
        
        Ok(())
    }

    // ========================================================================
    // Run Loop
    // ========================================================================

    pub async fn run_assignment_loop(&self) {
        loop {
            // Try to assign pending tasks
            match self.assign_next_task().await {
                Ok(Some((agent_id, task))) => {
                    debug!("Assigned task {:?} to agent {:?}", task.id, agent_id);
                }
                Ok(None) => {
                    // No tasks to assign or no workers available
                }
                Err(e) => {
                    warn!("Error assigning task: {}", e);
                }
            }

            // Sleep before next check
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}