use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use crate::mcp::error::{McpError, McpResult};
use crate::mcp::protocol::ToolDescription;
use crate::{
    channel::ChannelManager,
    packet::{Packet, Priority},
    batcher::FrameBatcher,
};

/// Tool trait for UI operations that Claude Code calls to update the browser
#[async_trait]
pub trait UiTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value>;
}

/// Context for UI tools to access server infrastructure
pub struct UiContext {
    pub channel_manager: Arc<ChannelManager>,
    pub batcher: Arc<FrameBatcher>,
}

/// Provides UI tools that Claude Code can call to update the browser
pub struct UiToolProvider {
    tools: HashMap<String, Arc<dyn UiTool>>,
}

impl UiToolProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            tools: HashMap::new(),
        };
        
        // Register UI tools that Claude Code calls to update browser
        provider.register(Arc::new(ShowFileTool));
        provider.register(Arc::new(UpdateEditorTool));
        provider.register(Arc::new(ShowTerminalOutputTool));
        provider.register(Arc::new(UpdateFileTreeTool));
        provider.register(Arc::new(ShowDiffTool));
        provider.register(Arc::new(ShowErrorTool));
        provider.register(Arc::new(UpdateStatusBarTool));
        provider.register(Arc::new(ShowNotificationTool));
        provider.register(Arc::new(OpenPanelTool));
        provider.register(Arc::new(ShowChatMessageTool));
        
        provider
    }

    pub fn register(&mut self, tool: Arc<dyn UiTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn UiTool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_tools(&self) -> Vec<ToolDescription> {
        self.tools
            .values()
            .map(|tool| ToolDescription {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.input_schema(),
            })
            .collect()
    }
}

// --- UI Tools that Claude Code calls ---

/// Show file content in the editor
struct ShowFileTool;

#[async_trait]
impl UiTool for ShowFileTool {
    fn name(&self) -> &str {
        "show_file"
    }

    fn description(&self) -> &str {
        "Display file content in the browser editor"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to display"
                },
                "content": {
                    "type": "string",
                    "description": "File content to show"
                },
                "language": {
                    "type": "string",
                    "description": "Language for syntax highlighting"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let path = arguments["path"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("path required".into()))?;
        let content = arguments["content"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("content required".into()))?;
        let language = arguments["language"].as_str().unwrap_or("text");
        
        // Send to UI channel (e.g., channel 10)
        let ui_message = json!({
            "type": "show_file",
            "path": path,
            "content": content,
            "language": language
        });
        
        let packet = Packet::new(
            10, // UI channel
            0,  // packet type
            Priority::High,
            Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({
            "success": true,
            "message": format!("Displayed {} in editor", path)
        }))
    }
}

/// Update the editor content
struct UpdateEditorTool;

#[async_trait]
impl UiTool for UpdateEditorTool {
    fn name(&self) -> &str {
        "update_editor"
    }

    fn description(&self) -> &str {
        "Update the current editor content in the browser"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "New content for the editor"
                },
                "cursor_position": {
                    "type": "object",
                    "properties": {
                        "line": {"type": "number"},
                        "column": {"type": "number"}
                    }
                },
                "selection": {
                    "type": "object",
                    "properties": {
                        "start": {
                            "type": "object",
                            "properties": {
                                "line": {"type": "number"},
                                "column": {"type": "number"}
                            }
                        },
                        "end": {
                            "type": "object",
                            "properties": {
                                "line": {"type": "number"},
                                "column": {"type": "number"}
                            }
                        }
                    }
                }
            },
            "required": ["content"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let content = arguments["content"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("content required".into()))?;
        
        let ui_message = json!({
            "type": "update_editor",
            "content": content,
            "cursor_position": arguments.get("cursor_position"),
            "selection": arguments.get("selection")
        });
        
        let packet = Packet::new(
            10, // UI channel
            0,
            Priority::High,
            Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({"success": true}))
    }
}

/// Show terminal output
struct ShowTerminalOutputTool;

#[async_trait]
impl UiTool for ShowTerminalOutputTool {
    fn name(&self) -> &str {
        "show_terminal_output"
    }

    fn description(&self) -> &str {
        "Display output in the browser terminal panel"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "output": {
                    "type": "string",
                    "description": "Terminal output to display"
                },
                "stream": {
                    "type": "string",
                    "enum": ["stdout", "stderr"],
                    "description": "Output stream type"
                }
            },
            "required": ["output"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let output = arguments["output"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("output required".into()))?;
        let stream = arguments["stream"].as_str().unwrap_or("stdout");
        
        let ui_message = json!({
            "type": "terminal_output",
            "output": output,
            "stream": stream
        });
        
        let packet = Packet::new(
            10, // UI channel
            0,
            Priority::Medium,
            Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({"success": true}))
    }
}

/// Update file tree
struct UpdateFileTreeTool;

#[async_trait]
impl UiTool for UpdateFileTreeTool {
    fn name(&self) -> &str {
        "update_file_tree"
    }

    fn description(&self) -> &str {
        "Update the file tree display in the browser"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path"
                },
                "entries": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "type": {"type": "string", "enum": ["file", "directory"]},
                            "size": {"type": "number"}
                        }
                    }
                }
            },
            "required": ["path", "entries"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let path = arguments["path"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("path required".into()))?;
        let entries = arguments["entries"].as_array()
            .ok_or_else(|| McpError::InvalidParameters("entries required".into()))?;
        
        let ui_message = json!({
            "type": "update_file_tree",
            "path": path,
            "entries": entries
        });
        
        let packet = Packet::new(
            10, // UI channel
            0,
            Priority::Low,
            Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({"success": true}))
    }
}

/// Show diff in the browser
struct ShowDiffTool;

#[async_trait]
impl UiTool for ShowDiffTool {
    fn name(&self) -> &str {
        "show_diff"
    }

    fn description(&self) -> &str {
        "Display a diff view in the browser"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "File being diffed"
                },
                "old_content": {
                    "type": "string",
                    "description": "Original content"
                },
                "new_content": {
                    "type": "string",
                    "description": "Modified content"
                }
            },
            "required": ["file_path", "old_content", "new_content"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let file_path = arguments["file_path"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("file_path required".into()))?;
        let old_content = arguments["old_content"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("old_content required".into()))?;
        let new_content = arguments["new_content"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("new_content required".into()))?;
        
        let ui_message = json!({
            "type": "show_diff",
            "file_path": file_path,
            "old_content": old_content,
            "new_content": new_content
        });
        
        let packet = Packet::new(
            10, // UI channel
            0,
            Priority::High,
            Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({"success": true}))
    }
}

/// Show error in the browser
struct ShowErrorTool;

#[async_trait]
impl UiTool for ShowErrorTool {
    fn name(&self) -> &str {
        "show_error"
    }

    fn description(&self) -> &str {
        "Display an error message in the browser"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Error message"
                },
                "file": {
                    "type": "string",
                    "description": "File where error occurred"
                },
                "line": {
                    "type": "number",
                    "description": "Line number"
                },
                "column": {
                    "type": "number",
                    "description": "Column number"
                }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let message = arguments["message"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("message required".into()))?;
        
        let ui_message = json!({
            "type": "show_error",
            "message": message,
            "file": arguments.get("file"),
            "line": arguments.get("line"),
            "column": arguments.get("column")
        });
        
        let packet = Packet::new(
            10, // UI channel
            0,
            Priority::Critical,
            Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({"success": true}))
    }
}

/// Update status bar
struct UpdateStatusBarTool;

#[async_trait]
impl UiTool for UpdateStatusBarTool {
    fn name(&self) -> &str {
        "update_status_bar"
    }

    fn description(&self) -> &str {
        "Update the status bar in the browser"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Status message"
                },
                "type": {
                    "type": "string",
                    "enum": ["info", "warning", "error", "success"],
                    "description": "Status type"
                }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let message = arguments["message"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("message required".into()))?;
        let status_type = arguments["type"].as_str().unwrap_or("info");
        
        let ui_message = json!({
            "type": "update_status_bar",
            "message": message,
            "status_type": status_type
        });
        
        let packet = Packet::new(
            10, // UI channel
            0,
            Priority::Low,
            Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({"success": true}))
    }
}

/// Show notification
struct ShowNotificationTool;

#[async_trait]
impl UiTool for ShowNotificationTool {
    fn name(&self) -> &str {
        "show_notification"
    }

    fn description(&self) -> &str {
        "Display a notification in the browser"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Notification title"
                },
                "message": {
                    "type": "string",
                    "description": "Notification message"
                },
                "type": {
                    "type": "string",
                    "enum": ["info", "warning", "error", "success"],
                    "description": "Notification type"
                },
                "duration": {
                    "type": "number",
                    "description": "Duration in milliseconds"
                }
            },
            "required": ["title", "message"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let title = arguments["title"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("title required".into()))?;
        let message = arguments["message"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("message required".into()))?;
        
        let ui_message = json!({
            "type": "show_notification",
            "title": title,
            "message": message,
            "notification_type": arguments.get("type").and_then(|v| v.as_str()).unwrap_or("info"),
            "duration": arguments.get("duration").and_then(|v| v.as_u64()).unwrap_or(5000)
        });
        
        let packet = Packet::new(
            10, // UI channel
            0,
            Priority::Medium,
            Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({"success": true}))
    }
}

/// Open a panel in the IDE
struct OpenPanelTool;

#[async_trait]
impl UiTool for OpenPanelTool {
    fn name(&self) -> &str {
        "open_panel"
    }

    fn description(&self) -> &str {
        "Open a specific panel in the browser IDE"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "panel": {
                    "type": "string",
                    "enum": ["editor", "terminal", "file_tree", "search", "debug", "version_control"],
                    "description": "Panel to open"
                },
                "focus": {
                    "type": "boolean",
                    "description": "Whether to focus the panel"
                }
            },
            "required": ["panel"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let panel = arguments["panel"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("panel required".into()))?;
        let focus = arguments["focus"].as_bool().unwrap_or(true);
        
        let ui_message = json!({
            "type": "open_panel",
            "panel": panel,
            "focus": focus
        });
        
        let packet = Packet::new(
            10, // UI channel
            0,
            Priority::Medium,
            Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({"success": true}))
    }
}

/// Show chat message in conversation
struct ShowChatMessageTool;

#[async_trait]
impl UiTool for ShowChatMessageTool {
    fn name(&self) -> &str {
        "show_chat_message"
    }

    fn description(&self) -> &str {
        "Display a message in the chat conversation"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "role": {
                    "type": "string",
                    "enum": ["assistant", "user", "system"],
                    "description": "Message sender role"
                },
                "content": {
                    "type": "string",
                    "description": "Message content (supports markdown)"
                },
                "timestamp": {
                    "type": "string",
                    "description": "ISO timestamp"
                }
            },
            "required": ["role", "content"]
        })
    }

    async fn execute(&self, arguments: Value, context: &UiContext) -> McpResult<Value> {
        let role = arguments["role"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("role required".into()))?;
        let content = arguments["content"].as_str()
            .ok_or_else(|| McpError::InvalidParameters("content required".into()))?;
        
        // Send to chat-assistant channel (1050)
        let chat_message = json!({
            "type": "show_chat_message",
            "role": role,
            "content": content,
            "timestamp": arguments.get("timestamp")
        });
        
        let packet = Packet::new(
            1050, // chat-assistant channel
            0,
            Priority::High,
            Bytes::from(serde_json::to_vec(&chat_message).unwrap()),
        );
        
        context.batcher.queue_packet(packet).await;
        
        Ok(json!({"success": true}))
    }
}

impl Default for UiToolProvider {
    fn default() -> Self {
        Self::new()
    }
}