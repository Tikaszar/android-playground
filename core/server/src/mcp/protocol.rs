use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// MCP Protocol Messages - Universal for any LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpMessage {
    // Connection management
    Initialize {
        client_info: ClientInfo,
        protocol_version: String,
    },
    Initialized,
    Ping,
    Pong,
    Shutdown,

    // Tool discovery
    ListTools,
    ToolsList {
        tools: Vec<ToolDescription>,
    },

    // Tool execution
    ToolCall {
        id: String,
        tool: String,
        arguments: Value,
    },
    ToolResult {
        id: String,
        result: Value,
        error: Option<String>,
    },

    // Prompts and responses
    Prompt {
        id: String,
        content: String,
        context: Option<Vec<String>>, // File paths for context
    },
    Response {
        id: String,
        content: String,
        tool_calls: Vec<ToolCall>,
    },
    StreamingResponse {
        id: String,
        delta: String,
        done: bool,
    },

    // Session management
    CreateSession {
        session_id: Option<String>,
    },
    SessionCreated {
        session_id: String,
    },
    EndSession {
        session_id: String,
    },

    // Error
    Error {
        code: String,
        message: String,
        data: Option<Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,        // "claude", "gpt", "llama", etc.
    pub version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub input_schema: Value, // JSON Schema
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub tool: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: String,
    pub output: Value,
    pub error: Option<String>,
}

/// Request wrapper for HTTP POST from LLM clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub session_id: String,
    pub message: McpMessage,
}

/// Response wrapper for SSE events to LLM clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub session_id: String,
    pub message: McpMessage,
}

/// SSE Event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
    pub id: Option<String>,
}

impl SseEvent {
    pub fn from_message(message: McpMessage, session_id: String) -> Self {
        let response = McpResponse {
            session_id,
            message,
        };
        
        Self {
            event: "message".to_string(),
            data: serde_json::to_string(&response).unwrap_or_default(),
            id: Some(Uuid::new_v4().to_string()),
        }
    }

    pub fn heartbeat() -> Self {
        Self {
            event: "heartbeat".to_string(),
            data: "{}".to_string(),
            id: None,
        }
    }
}