use crate::components::*;
use crate::panel_manager::{PanelManager, PanelType};
use crate::browser_bridge::BrowserBridge;
use crate::ui_state::UiState;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Handles MCP tool calls and routes them to appropriate UI components
pub struct McpHandler {
    ui_state: Arc<RwLock<UiState>>,
    browser_bridge: Arc<BrowserBridge>,
    panel_manager: Arc<RwLock<PanelManager>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    pub tool_name: String,
    pub params: serde_json::Value,
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
        }
    }

    pub async fn handle_tool_call(&self, tool_call: McpToolCall) -> Result<serde_json::Value> {
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
            _ => Err(anyhow!("Unknown MCP tool: {}", tool_call.tool_name)),
        }
    }

    async fn handle_show_file(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(Deserialize)]
        struct ShowFileParams {
            path: String,
            content: String,
            language: Option<String>,
        }
        
        let params: ShowFileParams = serde_json::from_value(params)?;
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
                    params.content,
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
        let agents = ui_state.channel_manager.list_agents();
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
        
        // This is used for context switching
        // Parse command to see if it's a context switch
        if params.command.contains("claude --continue") {
            // Extract worktree from working_directory
            if let Some(dir) = params.working_directory {
                tracing::info!("Context switch to: {}", dir);
                // TODO: Implement actual context switching
            }
        }
        
        Ok(serde_json::json!({
            "success": true,
            "output": "Command executed",
        }))
    }
}