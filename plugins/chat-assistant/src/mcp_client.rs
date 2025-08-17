use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info, warn};

/// MCP Client for connecting to the MCP server
pub struct McpClient {
    server_url: String,
    session_id: Option<String>,
    connected: bool,
}

impl McpClient {
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            session_id: None,
            connected: false,
        }
    }

    /// Connect to MCP server and establish session
    pub async fn connect(&mut self) -> Result<(), String> {
        // Create session via HTTP POST
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/sessions", self.server_url))
            .send()
            .await
            .map_err(|e| format!("Failed to create session: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to create session: {}", response.status()));
        }

        let session_data: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse session response: {}", e))?;

        self.session_id = session_data["session_id"]
            .as_str()
            .map(|s| s.to_string());

        if self.session_id.is_none() {
            return Err("No session ID in response".to_string());
        }

        info!("Connected to MCP server with session: {:?}", self.session_id);
        
        // Initialize the connection
        self.send_initialize().await?;
        
        self.connected = true;
        Ok(())
    }

    /// Send initialization message
    async fn send_initialize(&self) -> Result<(), String> {
        let session_id = self.session_id.as_ref()
            .ok_or("No session ID")?;

        let init_message = serde_json::json!({
            "session_id": session_id,
            "message": {
                "type": "initialize",
                "client_info": {
                    "name": "android-playground",
                    "version": "0.1.0",
                    "capabilities": ["tools", "streaming"]
                },
                "protocol_version": "1.0"
            }
        });

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/message", self.server_url))
            .json(&init_message)
            .send()
            .await
            .map_err(|e| format!("Failed to send initialize: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Initialize failed: {}", response.status()));
        }

        Ok(())
    }

    /// Send a prompt to the connected LLM
    pub async fn send_prompt(&self, prompt: String, context_files: Vec<String>) -> Result<String, String> {
        if !self.connected {
            return Err("Not connected to MCP server".to_string());
        }

        let session_id = self.session_id.as_ref()
            .ok_or("No session ID")?;

        let prompt_message = serde_json::json!({
            "session_id": session_id,
            "message": {
                "type": "prompt",
                "id": uuid::Uuid::new_v4().to_string(),
                "content": prompt,
                "context": if context_files.is_empty() { None } else { Some(context_files) }
            }
        });

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/message", self.server_url))
            .json(&prompt_message)
            .send()
            .await
            .map_err(|e| format!("Failed to send prompt: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Prompt failed: {}", response.status()));
        }

        // In a real implementation, we'd listen to SSE for the response
        // For now, return a placeholder
        Ok("Response will arrive via SSE stream".to_string())
    }

    /// Execute a tool through MCP
    pub async fn execute_tool(&self, tool_name: String, arguments: Value) -> Result<Value, String> {
        if !self.connected {
            return Err("Not connected to MCP server".to_string());
        }

        let session_id = self.session_id.as_ref()
            .ok_or("No session ID")?;

        let tool_message = serde_json::json!({
            "session_id": session_id,
            "message": {
                "type": "tool_call",
                "id": uuid::Uuid::new_v4().to_string(),
                "tool": tool_name,
                "arguments": arguments
            }
        });

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/message", self.server_url))
            .json(&tool_message)
            .send()
            .await
            .map_err(|e| format!("Failed to execute tool: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Tool execution failed: {}", response.status()));
        }

        // Tool results will arrive via SSE
        Ok(serde_json::json!({"status": "executing"}))
    }

    /// Handle terminal commands
    pub async fn handle_command(&self, command: &str) -> Result<String, String> {
        match command {
            "/clear" => {
                // Clear conversation in UI
                Ok("Conversation cleared".to_string())
            }
            "/exit" => {
                // Disconnect from MCP
                Ok("Exiting...".to_string())
            }
            "/reset" => {
                // Reset context
                Ok("Context reset".to_string())
            }
            "/pwd" => {
                // Get working directory via tool
                self.execute_tool(
                    "execute_command".to_string(),
                    serde_json::json!({"command": "pwd"})
                ).await.map(|v| v.to_string())
            }
            "/ls" => {
                // List files via tool
                self.execute_tool(
                    "list_files".to_string(),
                    serde_json::json!({"path": "."})
                ).await.map(|v| v.to_string())
            }
            cmd if cmd.starts_with("/cd ") => {
                let path = cmd.strip_prefix("/cd ").unwrap_or(".");
                self.execute_tool(
                    "execute_command".to_string(),
                    serde_json::json!({"command": format!("cd {}", path)})
                ).await.map(|v| v.to_string())
            }
            _ => {
                Err(format!("Unknown command: {}", command))
            }
        }
    }

    /// List available tools from MCP server
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/tools", self.server_url))
            .send()
            .await
            .map_err(|e| format!("Failed to list tools: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to list tools: {}", response.status()));
        }

        let tools: Vec<ToolInfo> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse tools: {}", e))?;

        Ok(tools)
    }

    /// Disconnect from MCP server
    pub async fn disconnect(&mut self) -> Result<(), String> {
        if let Some(session_id) = &self.session_id {
            let client = reqwest::Client::new();
            let _ = client
                .delete(format!("{}/sessions/{}", self.server_url, session_id))
                .send()
                .await;
        }

        self.connected = false;
        self.session_id = None;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// SSE event handler for receiving messages from MCP server
pub struct SseHandler {
    session_id: String,
    event_source: Option<reqwest::Response>,
}

impl SseHandler {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            event_source: None,
        }
    }

    /// Start listening to SSE stream
    pub async fn connect(&mut self, server_url: &str) -> Result<(), String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/sse/{}", server_url, self.session_id))
            .send()
            .await
            .map_err(|e| format!("Failed to connect SSE: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("SSE connection failed: {}", response.status()));
        }

        self.event_source = Some(response);
        Ok(())
    }

    /// Process incoming SSE events
    pub async fn process_events<F>(&mut self, mut handler: F) -> Result<(), String>
    where
        F: FnMut(McpMessage),
    {
        let response = self.event_source.as_mut()
            .ok_or("No SSE connection")?;

        // This is simplified - in production we'd properly parse SSE format
        while let Ok(chunk) = response.chunk().await {
            if let Some(data) = chunk {
                if let Ok(text) = String::from_utf8(data.to_vec()) {
                    // Parse SSE event
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            if let Ok(message) = serde_json::from_str::<McpMessage>(&line[6..]) {
                                handler(message);
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpMessage {
    Response {
        id: String,
        content: String,
        tool_calls: Vec<Value>,
    },
    StreamingResponse {
        id: String,
        delta: String,
        done: bool,
    },
    ToolResult {
        id: String,
        result: Value,
        error: Option<String>,
    },
    Error {
        code: String,
        message: String,
        data: Option<Value>,
    },
}