use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use futures_util::stream::Stream;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tracing::{info, debug};
use serde_json::Value;

use crate::{
    packet::{Packet, Priority},
    websocket::WebSocketState,
};
use crate::mcp::{
    protocol::{McpMessage, McpRequest, SseEvent},
    session::SessionManager,
};

/// MCP server state shared across handlers
struct McpState {
    session_manager: Arc<SessionManager>,
}

/// MCP Server that integrates with WebSocketState
/// 
/// This server provides the MCP protocol for LLM integration.
/// It publishes tool calls to channels for plugins to handle.
/// 
/// Channel allocation:
/// - 2000: MCP tool calls (from LLM to plugins)
/// - 2001: MCP tool results (from plugins to LLM)
/// - 2002-2999: Reserved for individual LLM sessions
pub struct McpServer {
    session_manager: Arc<SessionManager>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new()),
        }
    }

    /// Create router for MCP endpoints
    pub fn router(&self) -> Router<Arc<WebSocketState>> {
        let mcp_state = Arc::new(McpState {
            session_manager: self.session_manager.clone(),
        });

        Router::new()
            // SSE endpoints for LLM clients to connect
            .route("/sse", get(sse_handler))
            .route("/sse/{session_id}", get(sse_handler_with_session))
            
            // HTTP endpoints for LLM responses
            .route("/message", post(handle_message))
            .route("/prompt", post(handle_prompt))
            
            // Session management
            .route("/sessions", get(list_sessions))
            .route("/sessions", post(create_session))
            .route("/sessions/{id}", axum::routing::delete(end_session))
            
            // Tool discovery - returns list of available tools
            // The actual tool list comes from plugins via channels
            .route("/tools", get(list_tools))
            
            // Health check
            .route("/health", get(health_check))
            
            .layer(axum::extract::Extension(mcp_state))
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// SSE handler for new connections
async fn sse_handler(
    State(ws_state): State<Arc<WebSocketState>>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (session_id, rx) = mcp_state.session_manager.create_session(None);
    info!("New MCP client connected: {}", session_id);
    
    // Allocate a channel for this LLM session (2002-2999)
    let channel_id = allocate_llm_channel(&session_id);
    let channel_name = format!("mcp-session-{}", session_id);
    let _ = ws_state.channel_manager.register_plugin(channel_name).await;
    
    // Notify plugins that a new LLM connected
    publish_mcp_event(
        ws_state.clone(),
        "llm_connected",
        serde_json::json!({
            "session_id": session_id.clone(),
            "channel_id": channel_id,
        }),
    ).await;
    
    sse_stream(session_id, rx, ws_state, mcp_state)
}

/// SSE handler with specific session ID
async fn sse_handler_with_session(
    Path(session_id): Path<String>,
    State(ws_state): State<Arc<WebSocketState>>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (_, rx) = mcp_state.session_manager.create_session(Some(session_id.clone()));
    info!("MCP client reconnected: {}", session_id);
    
    // Allocate a channel for this LLM session
    let channel_id = allocate_llm_channel(&session_id);
    let channel_name = format!("mcp-session-{}", session_id);
    let _ = ws_state.channel_manager.register_plugin(channel_name).await;
    
    // Notify plugins that an LLM reconnected
    publish_mcp_event(
        ws_state.clone(),
        "llm_reconnected",
        serde_json::json!({
            "session_id": session_id.clone(),
            "channel_id": channel_id,
        }),
    ).await;
    
    sse_stream(session_id, rx, ws_state, mcp_state)
}

/// Create SSE stream for a session
fn sse_stream(
    session_id: String,
    rx: mpsc::UnboundedReceiver<McpMessage>,
    _ws_state: Arc<WebSocketState>,
    _mcp_state: Arc<McpState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Convert McpMessage stream to SseEvent stream
    let stream = UnboundedReceiverStream::new(rx)
        .map(move |message| {
            // Convert McpMessage to SseEvent
            let sse_event = SseEvent::from_message(message, session_id.clone());
            
            // Convert to axum Event
            Ok(Event::default()
                .event(sse_event.event)
                .data(sse_event.data)
                .id(sse_event.id.unwrap_or_default()))
        });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("heartbeat"),
    )
}

/// Handle messages from LLM clients
async fn handle_message(
    State(ws_state): State<Arc<WebSocketState>>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    let session_id = &request.session_id;
    
    // Update session activity
    if let Err(e) = mcp_state.session_manager.update_session(session_id, |session| {
        session.update_activity();
    }) {
        return (StatusCode::NOT_FOUND, Json(json_error("Session not found", Some(e))));
    }
    
    // Process the message based on type
    match request.message {
        McpMessage::ToolCall { id, tool, arguments } => {
            // Publish tool call to channel 2000 for plugins to handle
            publish_mcp_event(
                ws_state.clone(),
                "tool_call",
                serde_json::json!({
                    "session_id": session_id.clone(),
                    "call_id": id,
                    "tool": tool,
                    "arguments": arguments,
                }),
            ).await;
            
            // The actual tool execution happens in plugins
            // They will send results back via channel 2001
            (StatusCode::OK, Json(json_success("Tool call forwarded to plugins")))
        },
        McpMessage::Response { id, content, tool_calls } => {
            // LLM is responding to a prompt - forward to plugins
            publish_mcp_event(
                ws_state.clone(),
                "llm_response",
                serde_json::json!({
                    "session_id": session_id.clone(),
                    "response_id": id,
                    "content": content,
                    "tool_calls": tool_calls,
                }),
            ).await;
            
            (StatusCode::OK, Json(json_success("Response forwarded to plugins")))
        },
        McpMessage::StreamingResponse { id, delta, done } => {
            // Forward streaming response to plugins
            publish_mcp_event(
                ws_state.clone(),
                "llm_streaming",
                serde_json::json!({
                    "session_id": session_id.clone(),
                    "response_id": id,
                    "delta": delta,
                    "done": done,
                }),
            ).await;
            
            (StatusCode::OK, Json(json_success("Streaming delta forwarded")))
        },
        _ => {
            (StatusCode::BAD_REQUEST, Json(json_error("Unsupported message type", None::<String>)))
        }
    }
}

/// Handle prompt requests - send prompt to LLM(s)
async fn handle_prompt(
    State(ws_state): State<Arc<WebSocketState>>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
    Json(request): Json<PromptRequest>,
) -> impl IntoResponse {
    let prompt_content = request.content;
    
    // Create a prompt message
    let prompt_message = McpMessage::Prompt {
        id: uuid::Uuid::new_v4().to_string(),
        content: prompt_content.clone(),
        context: request.context_files,
    };
    
    // If session_id provided, send to specific LLM
    if let Some(session_id) = request.session_id {
        // Get the session and send the message
        if let Some(session) = mcp_state.session_manager.get_session(&session_id) {
            if let Err(e) = session.send_message(prompt_message) {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error("Failed to send prompt", Some(e))));
            }
            
            // Notify plugins that a prompt was sent
            publish_mcp_event(
                ws_state.clone(),
                "prompt_sent",
                serde_json::json!({
                    "session_id": session_id,
                    "content": prompt_content,
                }),
            ).await;
            
            (StatusCode::OK, Json(json_success("Prompt sent to session")))
        } else {
            (StatusCode::NOT_FOUND, Json(json_error("Session not found", None::<String>)))
        }
    } else {
        // Broadcast to all active LLM sessions
        mcp_state.session_manager.broadcast_to_all(prompt_message);
        
        // Notify plugins that a prompt was broadcast
        publish_mcp_event(
            ws_state.clone(),
            "prompt_broadcast",
            serde_json::json!({
                "content": prompt_content,
            }),
        ).await;
        
        (StatusCode::OK, Json(json_success("Prompt broadcast to all sessions")))
    }
}

/// List all active sessions
async fn list_sessions(
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> impl IntoResponse {
    let sessions = mcp_state.session_manager.list_sessions();
    Json(serde_json::json!({
        "sessions": sessions
    }))
}

/// Create a new session
async fn create_session(
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> impl IntoResponse {
    let (session_id, _) = mcp_state.session_manager.create_session(None);
    Json(serde_json::json!({
        "session_id": session_id
    }))
}

/// End a specific session
async fn end_session(
    Path(id): Path<String>,
    State(ws_state): State<Arc<WebSocketState>>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> impl IntoResponse {
    mcp_state.session_manager.remove_session(&id);
    
    // Notify plugins that an LLM disconnected
    publish_mcp_event(
        ws_state,
        "llm_disconnected",
        serde_json::json!({
            "session_id": id,
        }),
    ).await;
    
    StatusCode::NO_CONTENT
}

/// List available tools
/// Tools are registered by plugins via channel messages
async fn list_tools(
    State(ws_state): State<Arc<WebSocketState>>,
) -> impl IntoResponse {
    // Request tool list from plugins via channel
    // For now, return a static list that plugins implement
    let tools = vec![
        serde_json::json!({
            "name": "show_file",
            "description": "Display file content in the browser editor",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to display"
                    },
                    "content": {
                        "type": "string",
                        "description": "File content"
                    }
                },
                "required": ["path", "content"]
            }
        }),
        serde_json::json!({
            "name": "update_editor",
            "description": "Update the current editor content",
            "input_schema": {
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "New content for the editor"
                    },
                    "cursor_position": {
                        "type": "object",
                        "properties": {
                            "line": {"type": "integer"},
                            "column": {"type": "integer"}
                        }
                    }
                },
                "required": ["content"]
            }
        }),
        serde_json::json!({
            "name": "show_terminal_output",
            "description": "Display output in the terminal",
            "input_schema": {
                "type": "object",
                "properties": {
                    "output": {
                        "type": "string",
                        "description": "Terminal output to display"
                    }
                },
                "required": ["output"]
            }
        }),
        serde_json::json!({
            "name": "update_file_tree",
            "description": "Update the file browser tree",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Root path for the tree"
                    },
                    "expanded": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of expanded paths"
                    }
                },
                "required": ["path"]
            }
        }),
        serde_json::json!({
            "name": "show_diff",
            "description": "Display a diff view",
            "input_schema": {
                "type": "object",
                "properties": {
                    "old_content": {
                        "type": "string",
                        "description": "Original content"
                    },
                    "new_content": {
                        "type": "string",
                        "description": "Modified content"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "File path for context"
                    }
                },
                "required": ["old_content", "new_content"]
            }
        }),
    ];
    
    // In the future, this could query plugins for their registered tools
    publish_mcp_event(
        ws_state,
        "tools_requested",
        serde_json::json!({}),
    ).await;
    
    Json(serde_json::json!({
        "tools": tools
    }))
}

/// Health check
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "mcp-server",
        "version": "0.1.0"
    }))
}

/// Publish an MCP event to channel 2000 for plugins to handle
async fn publish_mcp_event(
    ws_state: Arc<WebSocketState>,
    event_type: &str,
    data: Value,
) {
    debug!("Publishing MCP event: {} -> {:?}", event_type, data);
    
    let message = serde_json::json!({
        "type": event_type,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": data,
    });
    
    let packet = Packet::new(
        2000, // MCP events channel
        0,
        Priority::High,
        Bytes::from(serde_json::to_vec(&message).unwrap()),
    );
    
    ws_state.batcher.queue_packet(packet).await;
}

/// Allocate a channel for an LLM session
fn allocate_llm_channel(session_id: &str) -> u16 {
    // Channels 2002-2999 for individual LLM sessions
    // Simple hash-based allocation
    let hash = session_id.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
    2002 + (hash % 998) as u16
}

// Request types
#[derive(serde::Deserialize)]
struct PromptRequest {
    content: String,
    context_files: Option<Vec<String>>,
    session_id: Option<String>,
}

// Helper functions
fn json_success(message: &str) -> serde_json::Value {
    serde_json::json!({
        "success": true,
        "message": message
    })
}

fn json_error<T: ToString>(message: &str, details: Option<T>) -> serde_json::Value {
    let mut response = serde_json::json!({
        "success": false,
        "error": message
    });
    
    if let Some(d) = details {
        response["details"] = serde_json::Value::String(d.to_string());
    }
    
    response
}