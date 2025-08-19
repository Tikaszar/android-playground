use crate::components::*;
use crate::panel_manager::{PanelManager, PanelType};
use crate::browser_bridge::BrowserBridge;
use crate::ui_state::UiState;
use crate::orchestrator::Orchestrator;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::path::PathBuf;
use tokio::fs;

/// Handles MCP tool calls and routes them to appropriate UI components
pub struct McpHandler {
    ui_state: Arc<RwLock<UiState>>,
    browser_bridge: Arc<BrowserBridge>,
    panel_manager: Arc<RwLock<PanelManager>>,
    orchestrator: Option<Arc<RwLock<Orchestrator>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    pub tool_name: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub message: String,
}

impl McpHandler {
    pub fn new(
        ui_state: Arc<RwLock<UiState>>,
        browser_bridge: Arc<BrowserBridge>,
        panel_manager: Arc<RwLock<PanelManager>>,
    ) -> Self {
        Self {
            ui_state,
            browser_bridge,
            panel_manager,
            orchestrator: None,
        }
    }
    
    pub fn set_orchestrator(&mut self, orchestrator: Arc<RwLock<Orchestrator>>) {
        self.orchestrator = Some(orchestrator);
    }

    pub async fn handle_tool_call(&self, tool_name: &str, params: serde_json::Value) -> Result<ToolResult> {
        let tool_call = McpToolCall {
            tool_name: tool_name.to_string(),
            params,
        };
        
        let result = self.handle_tool_call_internal(tool_call).await?;
        
        Ok(ToolResult {
            success: true,
            message: result.to_string(),
        })
    }
    
    async fn handle_tool_call_internal(&self, tool_call: McpToolCall) -> Result<serde_json::Value> {
        tracing::debug!("Handling MCP tool call: {}", tool_call.tool_name);
        
        match tool_call.tool_name.as_str() {
            "show_file" => self.handle_show_file(tool_call.params).await,
            "update_editor" => self.handle_update_editor(tool_call.params).await,
            "show_terminal_output" => self.handle_show_terminal_output(tool_call.params).await,
            "update_file_tree" => self.handle_update_file_tree(tool_call.params).await,
            "show_diff" => self.handle_show_diff(tool_call.params).await,
            "show_error" => self.handle_show_error(tool_call.params).await,
            "update_status_bar" => self.handle_update_status_bar(tool_call.params).await,
            "show_notification" => self.handle_show_notification(tool_call.params).await,
            "show_chat_message" => self.handle_show_chat_message(tool_call.params).await,
            "execute_command" => self.handle_execute_command(tool_call.params).await,
            "save_file" => self.handle_save_file(tool_call.params).await,
            "read_file" => self.handle_read_file(tool_call.params).await,
            "create_task" => self.handle_create_task(tool_call.params).await,
            "create_worker" => self.handle_create_worker(tool_call.params).await,
            "assign_task" => self.handle_assign_task(tool_call.params).await,
            "complete_task" => self.handle_complete_task(tool_call.params).await,
            _ => Err(anyhow!("Unknown MCP tool: {}", tool_call.tool_name)),
        }
    }

    async fn handle_show_file(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct ShowFileParams {
            path: String,
            content: Option<String>,  // Optional - if not provided, read from disk
            language: Option<String>,
        }
        
        let mut params: ShowFileParams = serde_json::from_value(params)?;
        
        // If content not provided, read from file system
        if params.content.is_none() {
            let path = PathBuf::from(&params.path);
            if path.exists() {
                params.content = Some(fs::read_to_string(&path).await?);
            } else {
                return Err(anyhow!("File not found: {}", params.path));
            }
        }
        
        let content = params.content.unwrap();
        let language = params.language.unwrap_or_else(|| {
            // Detect language from file extension
            let ext = std::path::Path::new(&params.path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            
            match ext {
                "rs" => "rust",
                "js" => "javascript",
                "ts" => "typescript",
                "py" => "python",
                "go" => "go",
                "html" => "html",
                "css" => "css",
                "json" => "json",
                "md" => "markdown",
                _ => "text",
            }.to_string()
        });
        
        let file_path = params.path.clone();
        
        // Send inline editor to active channel
        let ui_state = self.ui_state.read().await;
        if let Some(channel_id) = ui_state.active_channel {
            // Use system agent for MCP-generated messages
            let system_agent = AgentId(Uuid::nil());
            
            ui_state.message_system
                .send_inline_editor(
                    channel_id,
                    system_agent,
                    params.path,
                    content,
                    language,
                )
                .await?;
        }
        
        // Also create/update editor panel
        let mut panel_manager = self.panel_manager.write().await;
        let panel_id = panel_manager.create_panel(
            PanelType::Editor,
            format!("Editor: {}", file_path),
        );
        panel_manager.show_panel(panel_id)?;
        
        Ok(serde_json::json!({
            "success": true,
            "panel_id": panel_id,
        }))
    }

    async fn handle_update_editor(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct UpdateEditorParams {
            content: String,
            cursor_position: Option<(usize, usize)>,
        }
        
        let params: UpdateEditorParams = serde_json::from_value(params)?;
        
        // Update the current editor content
        // This would update the InlineEditor component in the active message
        
        Ok(serde_json::json!({
            "success": true,
        }))
    }

    async fn handle_show_terminal_output(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct ShowTerminalParams {
            output: String,
            session_id: Option<String>,
        }
        
        let params: ShowTerminalParams = serde_json::from_value(params)?;
        let session_id = params.session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Send inline terminal to active channel
        let ui_state = self.ui_state.read().await;
        if let Some(channel_id) = ui_state.active_channel {
            let system_agent = AgentId(Uuid::nil());
            let output_lines: Vec<String> = params.output.lines().map(|s| s.to_string()).collect();
            
            ui_state.message_system
                .send_inline_terminal(
                    channel_id,
                    system_agent,
                    session_id.clone(),
                    output_lines,
                )
                .await?;
        }
        
        Ok(serde_json::json!({
            "success": true,
            "session_id": session_id,
        }))
    }

    async fn handle_update_file_tree(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct UpdateFileTreeParams {
            path: String,
            entries: Vec<FileEntryParam>,
        }
        
        #[derive(Deserialize)]
        struct FileEntryParam {
            name: String,
            path: String,
            is_directory: bool,
            size: Option<u64>,
        }
        
        let params: UpdateFileTreeParams = serde_json::from_value(params)?;
        
        // Convert to our FileEntry type
        let entries: Vec<FileEntry> = params.entries.into_iter().map(|e| FileEntry {
            name: e.name,
            path: std::path::PathBuf::from(e.path),
            is_directory: e.is_directory,
            size: e.size,
            git_status: None, // TODO: Add git status detection
        }).collect();
        
        // Send inline file browser to active channel
        let ui_state = self.ui_state.read().await;
        if let Some(channel_id) = ui_state.active_channel {
            let system_agent = AgentId(Uuid::nil());
            
            ui_state.message_system
                .send_inline_file_browser(
                    channel_id,
                    system_agent,
                    std::path::PathBuf::from(params.path),
                    entries,
                )
                .await?;
        }
        
        Ok(serde_json::json!({
            "success": true,
        }))
    }

    async fn handle_show_diff(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct ShowDiffParams {
            file_path: String,
            diff: String, // Unified diff format
        }
        
        let params: ShowDiffParams = serde_json::from_value(params)?;
        
        // Parse the diff into hunks
        // For now, just create a simple hunk
        let hunks = vec![]; // TODO: Parse actual diff
        
        // Send inline diff to active channel
        let ui_state = self.ui_state.read().await;
        if let Some(channel_id) = ui_state.active_channel {
            let system_agent = AgentId(Uuid::nil());
            
            ui_state.message_system
                .send_inline_diff(
                    channel_id,
                    system_agent,
                    params.file_path,
                    hunks,
                )
                .await?;
        }
        
        Ok(serde_json::json!({
            "success": true,
        }))
    }

    async fn handle_show_error(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct ShowErrorParams {
            message: String,
        }
        
        let params: ShowErrorParams = serde_json::from_value(params)?;
        
        // Send error as system notification
        let ui_state = self.ui_state.read().await;
        if let Some(channel_id) = ui_state.active_channel {
            ui_state.message_system
                .send_system_notification(
                    channel_id,
                    format!("❌ Error: {}", params.message),
                )
                .await?;
        }
        
        Ok(serde_json::json!({
            "success": true,
        }))
    }

    async fn handle_update_status_bar(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct UpdateStatusParams {
            message: String,
        }
        
        let params: UpdateStatusParams = serde_json::from_value(params)?;
        
        // Update browser status bar via bridge
        self.browser_bridge.update_status(params.message).await?;
        
        Ok(serde_json::json!({
            "success": true,
        }))
    }

    async fn handle_show_notification(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct ShowNotificationParams {
            title: String,
            message: String,
        }
        
        let params: ShowNotificationParams = serde_json::from_value(params)?;
        
        // Send notification to browser
        self.browser_bridge.show_notification(params.title, params.message).await?;
        
        Ok(serde_json::json!({
            "success": true,
        }))
    }

    async fn handle_show_chat_message(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct ShowChatParams {
            sender: String,
            message: String,
        }
        
        let params: ShowChatParams = serde_json::from_value(params)?;
        
        // Find or create agent for sender
        let ui_state = self.ui_state.read().await;
        let agents = ui_state.channel_manager.read().await.list_agents();
        let agent_id = agents
            .iter()
            .find(|a| a.name == params.sender)
            .map(|a| a.id)
            .unwrap_or(AgentId(Uuid::nil()));
        
        // Send message to active channel
        if let Some(channel_id) = ui_state.active_channel {
            ui_state.message_system
                .send_text_message(channel_id, agent_id, params.message)
                .await?;
        }
        
        Ok(serde_json::json!({
            "success": true,
        }))
    }

    async fn handle_execute_command(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct ExecuteCommandParams {
            command: String,
            working_directory: Option<String>,
        }
        
        let params: ExecuteCommandParams = serde_json::from_value(params)?;
        
        // This is used for context switching and git operations
        use tokio::process::Command;
        
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&params.command);
        
        if let Some(dir) = &params.working_directory {
            cmd.current_dir(dir);
        }
        
        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Parse command to see if it's a context switch
        if params.command.contains("claude --continue") {
            // Extract worktree from working_directory
            if let Some(dir) = params.working_directory {
                tracing::info!("Context switch to: {}", dir);
                
                // Update UI state with new context
                let ui_state = self.ui_state.write().await;
                // TODO: Add current_worktree field to ui_state
                
                // Send notification
                self.browser_bridge.show_notification(
                    "Context Switched".to_string(),
                    format!("Switched to worktree: {}", dir)
                ).await?;
            }
        }
        
        // Handle git worktree commands
        if params.command.starts_with("git worktree") {
            tracing::info!("Git worktree command: {}", params.command);
        }
        
        Ok(serde_json::json!({
            "success": output.status.success(),
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code().unwrap_or(-1),
        }))
    }
    
    async fn handle_save_file(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct SaveFileParams {
            path: String,
            content: String,
            create_directories: Option<bool>,
        }
        
        let params: SaveFileParams = serde_json::from_value(params)?;
        let path = PathBuf::from(&params.path);
        
        // Create parent directories if requested
        if params.create_directories.unwrap_or(false) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
        }
        
        // Write file to disk
        fs::write(&path, &params.content).await?;
        
        tracing::info!("Saved file: {}", params.path);
        
        // Send notification to browser
        self.browser_bridge.show_notification(
            "File Saved".to_string(),
            format!("Successfully saved {}", params.path)
        ).await?;
        
        Ok(serde_json::json!({
            "success": true,
            "path": params.path,
            "bytes_written": params.content.len(),
        }))
    }
    
    async fn handle_read_file(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct ReadFileParams {
            path: String,
        }
        
        let params: ReadFileParams = serde_json::from_value(params)?;
        let path = PathBuf::from(&params.path);
        
        // Check if file exists
        if !path.exists() {
            return Err(anyhow!("File not found: {}", params.path));
        }
        
        // Read file content
        let content = fs::read_to_string(&path).await?;
        
        tracing::info!("Read file: {} ({} bytes)", params.path, content.len());
        
        Ok(serde_json::json!({
            "success": true,
            "path": params.path,
            "content": content,
            "size": content.len(),
        }))
    }
    
    async fn handle_create_task(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct CreateTaskParams {
            title: String,
            description: String,
            priority: Option<String>,
            context_files: Option<Vec<String>>,
        }
        
        let params: CreateTaskParams = serde_json::from_value(params)?;
        
        let priority = match params.priority.as_deref() {
            Some("critical") => TaskPriority::Critical,
            Some("high") => TaskPriority::High,
            Some("low") => TaskPriority::Low,
            _ => TaskPriority::Medium,
        };
        
        let context_files: Vec<PathBuf> = params.context_files
            .unwrap_or_default()
            .into_iter()
            .map(PathBuf::from)
            .collect();
        
        if let Some(orchestrator) = &self.orchestrator {
            let orchestrator = orchestrator.read().await;
            let task_id = orchestrator.create_task(
                params.title.clone(),
                params.description,
                priority,
                context_files,
            ).await?;
            
            Ok(serde_json::json!({
                "success": true,
                "task_id": task_id,
                "title": params.title,
            }))
        } else {
            Err(anyhow!("Orchestrator not initialized"))
        }
    }
    
    async fn handle_create_worker(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct CreateWorkerParams {
            name: String,
            worktree_path: Option<String>,
        }
        
        let params: CreateWorkerParams = serde_json::from_value(params)?;
        
        let worktree_path = if let Some(path) = params.worktree_path {
            PathBuf::from(path)
        } else {
            // Create default worktree path
            PathBuf::from(format!("/data/data/com.termux/files/home/android-playground-worker-{}", 
                params.name.to_lowercase().replace(' ', "-")))
        };
        
        if let Some(orchestrator) = &self.orchestrator {
            let orchestrator = orchestrator.read().await;
            let agent_id = orchestrator.create_worker(params.name.clone(), worktree_path).await?;
            
            Ok(serde_json::json!({
                "success": true,
                "agent_id": agent_id,
                "name": params.name,
            }))
        } else {
            Err(anyhow!("Orchestrator not initialized"))
        }
    }
    
    async fn handle_assign_task(&self, _params: serde_json::Value) -> Result<serde_json::Value> {
        if let Some(orchestrator) = &self.orchestrator {
            let orchestrator = orchestrator.read().await;
            
            if let Some((agent_id, task)) = orchestrator.assign_next_task().await? {
                Ok(serde_json::json!({
                    "success": true,
                    "agent_id": agent_id,
                    "task_id": task.id,
                    "task_title": task.title,
                }))
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "message": "No tasks to assign or no workers available",
                }))
            }
        } else {
            Err(anyhow!("Orchestrator not initialized"))
        }
    }
    
    async fn handle_complete_task(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct CompleteTaskParams {
            agent_id: String,
            task_id: String,
            success: bool,
            output: String,
            files_modified: Option<Vec<String>>,
            duration_seconds: Option<u64>,
        }
        
        let params: CompleteTaskParams = serde_json::from_value(params)?;
        
        let agent_id = AgentId(Uuid::parse_str(&params.agent_id)?);
        let task_id = TaskId(Uuid::parse_str(&params.task_id)?);
        
        let result = TaskResult {
            task_id,
            success: params.success,
            output: params.output,
            files_modified: params.files_modified
                .unwrap_or_default()
                .into_iter()
                .map(PathBuf::from)
                .collect(),
            duration_seconds: params.duration_seconds.unwrap_or(0),
        };
        
        if let Some(orchestrator) = &self.orchestrator {
            let orchestrator = orchestrator.read().await;
            orchestrator.complete_task(agent_id, result).await?;
            
            Ok(serde_json::json!({
                "success": true,
            }))
        } else {
            Err(anyhow!("Orchestrator not initialized"))
        }
    }
}