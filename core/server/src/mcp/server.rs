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
use tracing::{error, info, warn, debug};

use crate::{
    channel::ChannelManager,
    packet::{Packet, Priority},
    batcher::FrameBatcher,
    websocket::WebSocketState,
};
use crate::mcp::{
    error::{McpError, McpResult},
    protocol::{McpMessage, McpRequest, McpResponse, SseEvent, ClientInfo},
    session::{SessionManager, SessionId},
    ui_tools::{UiToolProvider, UiContext},
};

/// MCP Server deeply integrated with core/server infrastructure
pub struct McpServer {
    session_manager: Arc<SessionManager>,
    ui_tool_provider: Arc<UiToolProvider>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new()),
            ui_tool_provider: Arc::new(UiToolProvider::new()),
        }
    }

    /// Build router that integrates with the main server's WebSocketState
    pub fn build_router(self) -> Router<Arc<WebSocketState>> {
        let mcp_state = Arc::new(McpState {
            session_manager: self.session_manager,
            ui_tool_provider: self.ui_tool_provider,
        });

        Router::new()
            // SSE endpoints for LLM clients to connect
            .route("/sse", get(sse_handler))
            .route("/sse/:session_id", get(sse_handler_with_session))
            
            // HTTP endpoints for LLM responses
            .route("/message", post(handle_message))
            .route("/prompt", post(handle_prompt))
            
            // Session management
            .route("/sessions", get(list_sessions))
            .route("/sessions", post(create_session))
            .route("/sessions/:id", axum::routing::delete(end_session))
            
            // Tool discovery - UI tools that Claude Code can call
            .route("/tools", get(list_tools))
            
            // Health check
            .route("/health", get(health_check))
            
            .layer(axum::extract::Extension(mcp_state))
    }
}

#[derive(Clone)]
struct McpState {
    session_manager: Arc<SessionManager>,
    ui_tool_provider: Arc<UiToolProvider>,
}

/// SSE handler - LLM clients connect here to receive prompts
async fn sse_handler(
    State(ws_state): State<Arc<WebSocketState>>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (session_id, rx) = mcp_state.session_manager.create_session(None);
    info!("New MCP client connected: {}", session_id);
    
    // Register this LLM session with a channel
    let channel_name = format!("llm-{}", session_id);
    let _ = ws_state.channel_manager.register_plugin(channel_name).await;
    
    sse_stream(session_id, rx, ws_state, mcp_state)
}

/// SSE handler with specific session ID
async fn sse_handler_with_session(
    Path(session_id): Path<String>,
    State(ws_state): State<Arc<WebSocketState>>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (_, rx) = mcp_state.session_manager.create_session(Some(session_id.clone()));
    info!("MCP client reconnected with session ID: {}", session_id);
    
    // Register with channel
    let channel_name = format!("llm-{}", session_id);
    let _ = ws_state.channel_manager.register_plugin(channel_name).await;
    
    sse_stream(session_id, rx, ws_state, mcp_state)
}

/// Create SSE stream
fn sse_stream(
    session_id: SessionId,
    rx: mpsc::UnboundedReceiver<McpMessage>,
    ws_state: Arc<WebSocketState>,
    mcp_state: Arc<McpState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Send initial tools list (UI tools that Claude Code can call)
    if let Some(session) = mcp_state.session_manager.get_session(&session_id) {
        let tools_message = McpMessage::ToolsList {
            tools: mcp_state.ui_tool_provider.list_tools(),
        };
        let _ = session.send_message(tools_message);
    }
    
    let stream = UnboundedReceiverStream::new(rx);
    
    let sse_stream = stream.map(move |message| {
        let event = SseEvent::from_message(message, session_id.clone());
        Ok(Event::default()
            .event(event.event)
            .data(event.data)
            .id(event.id.unwrap_or_default()))
    });
    
    Sse::new(sse_stream).keep_alive(
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
    
    match request.message {
        McpMessage::Initialize { client_info, .. } => {
            mcp_state.session_manager.update_session(session_id, |session| {
                session.set_client_info(client_info.clone());
            }).ok();
            
            // Track which LLM is connected
            info!("LLM client initialized: {} v{}", client_info.name, client_info.version);
            
            // Send initialized response
            if let Some(session) = mcp_state.session_manager.get_session(session_id) {
                let _ = session.send_message(McpMessage::Initialized);
            }
            
            (StatusCode::OK, Json(json_success("Initialized")))
        }
        
        McpMessage::ToolCall { id, tool, arguments } => {
            // Execute UI tool that Claude Code is calling
            let ws_state_clone = ws_state.clone();
            let mcp_state_clone = mcp_state.clone();
            let session_id_clone = session_id.clone();
            let tool_id = id.clone();
            
            tokio::spawn(async move {
                execute_ui_tool(
                    ws_state_clone, 
                    mcp_state_clone, 
                    session_id_clone, 
                    tool_id, 
                    tool, 
                    arguments
                ).await;
            });
            
            (StatusCode::ACCEPTED, Json(json_success("Tool execution started")))
        }
        
        McpMessage::Response { id, content, tool_calls } => {
            // Claude Code's response to a prompt - forward to chat UI
            let tool_calls_json: Vec<serde_json::Value> = tool_calls
                .into_iter()
                .map(|tc| serde_json::to_value(tc).unwrap_or(serde_json::Value::Null))
                .collect();
            forward_response_to_ui(ws_state.clone(), id, content, tool_calls_json).await;
            (StatusCode::OK, Json(json_success("Response forwarded to UI")))
        }
        
        McpMessage::StreamingResponse { id, delta, done } => {
            // Streaming response from Claude Code
            forward_streaming_to_ui(ws_state.clone(), id, delta, done).await;
            (StatusCode::OK, Json(json_success("Streaming response forwarded")))
        }
        
        _ => {
            (StatusCode::OK, Json(json_success("Message processed")))
        }
    }
}

/// Handle prompts from the browser UI to send to Claude Code
async fn handle_prompt(
    State(ws_state): State<Arc<WebSocketState>>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
    Json(prompt_request): Json<PromptRequest>,
) -> impl IntoResponse {
    // Find the target LLM session or broadcast to all
    let prompt_message = McpMessage::Prompt {
        id: uuid::Uuid::new_v4().to_string(),
        content: prompt_request.content,
        context: prompt_request.context_files,
    };
    
    if let Some(session_id) = prompt_request.session_id {
        // Send to specific LLM
        if let Some(session) = mcp_state.session_manager.get_session(&session_id) {
            let _ = session.send_message(prompt_message);
            (StatusCode::OK, Json(json_success("Prompt sent to LLM")))
        } else {
            (StatusCode::NOT_FOUND, Json(json_error("Session not found", None::<String>)))
        }
    } else {
        // Broadcast to all connected LLMs
        mcp_state.session_manager.broadcast_to_all(prompt_message);
        (StatusCode::OK, Json(json_success("Prompt broadcast to all LLMs")))
    }
}

/// Execute a UI tool and send result back to LLM
async fn execute_ui_tool(
    ws_state: Arc<WebSocketState>,
    mcp_state: Arc<McpState>,
    session_id: String,
    tool_id: String,
    tool_name: String,
    arguments: serde_json::Value,
) {
    let ui_context = UiContext {
        channel_manager: ws_state.channel_manager.clone(),
        batcher: ws_state.batcher.clone(),
    };
    
    let result = if let Some(tool) = mcp_state.ui_tool_provider.get_tool(&tool_name) {
        match tool.execute(arguments, &ui_context).await {
            Ok(output) => McpMessage::ToolResult {
                id: tool_id,
                result: output,
                error: None,
            },
            Err(e) => McpMessage::ToolResult {
                id: tool_id,
                result: serde_json::Value::Null,
                error: Some(e.to_string()),
            },
        }
    } else {
        McpMessage::ToolResult {
            id: tool_id,
            result: serde_json::Value::Null,
            error: Some(format!("Tool '{}' not found", tool_name)),
        }
    };
    
    if let Some(session) = mcp_state.session_manager.get_session(&session_id) {
        let _ = session.send_message(result);
    }
}

/// Forward Claude Code's response to the UI
async fn forward_response_to_ui(
    ws_state: Arc<WebSocketState>,
    id: String,
    content: String,
    tool_calls: Vec<serde_json::Value>,
) {
    debug!("Forwarding response to UI: {}", id);
    
    // Send to chat-assistant channel (1050)
    let ui_message = serde_json::json!({
        "type": "llm_response",
        "id": id,
        "content": content,
        "tool_calls": tool_calls,
    });
    
    let packet = Packet::new(
        1050, // chat-assistant channel
        0,
        Priority::High,
        Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
    );
    
    ws_state.batcher.queue_packet(packet).await;
}

/// Forward streaming response to UI
async fn forward_streaming_to_ui(
    ws_state: Arc<WebSocketState>,
    id: String,
    delta: String,
    done: bool,
) {
    debug!("Forwarding streaming response to UI: {} (done: {})", id, done);
    
    let ui_message = serde_json::json!({
        "type": "llm_streaming",
        "id": id,
        "delta": delta,
        "done": done,
    });
    
    let packet = Packet::new(
        1050, // chat-assistant channel
        0,
        Priority::High,
        Bytes::from(serde_json::to_vec(&ui_message).unwrap()),
    );
    
    ws_state.batcher.queue_packet(packet).await;
}

/// Allocate a channel for an LLM session
fn allocate_llm_channel(session_id: &str) -> u16 {
    // Channels 2000-2999 for LLM sessions
    // Simple hash-based allocation
    let hash = session_id.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
    2000 + (hash % 1000) as u16
}

/// List active sessions
async fn list_sessions(
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> impl IntoResponse {
    let sessions = mcp_state.session_manager.list_sessions();
    Json(sessions)
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

/// End a session
async fn end_session(
    Path(id): Path<String>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> impl IntoResponse {
    if mcp_state.session_manager.remove_session(&id).is_some() {
        (StatusCode::OK, Json(json_success("Session ended")))
    } else {
        (StatusCode::NOT_FOUND, Json(json_error("Session not found", None::<String>)))
    }
}

/// List available UI tools
async fn list_tools(
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> impl IntoResponse {
    Json(mcp_state.ui_tool_provider.list_tools())
}

/// Health check
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "mcp-server",
        "version": "0.1.0"
    }))
}

// Request types
#[derive(serde::Deserialize)]
struct PromptRequest {
    content: String,
    context_files: Option<Vec<String>>,
    session_id: Option<String>, // Optional - if not provided, broadcast to all
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