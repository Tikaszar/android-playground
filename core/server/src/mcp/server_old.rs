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
use futures_util::stream::Stream;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

use crate::mcp::{
    error::{McpError, McpResult},
    protocol::{McpMessage, McpRequest, McpResponse, SseEvent, ToolCall},
    session::{SessionManager, SessionId},
    tools::ToolProvider,
};

/// MCP Server - Provides tools to any LLM (Claude Code, GPT, Llama, etc.)
pub struct McpServer {
    session_manager: Arc<SessionManager>,
    tool_provider: Arc<ToolProvider>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new()),
            tool_provider: Arc::new(ToolProvider::new()),
        }
    }

    /// Build router to be integrated into main server
    pub fn build_router<S>(self) -> Router<S> 
    where
        S: Clone + Send + Sync + 'static,
    {
        let state = AppState {
            session_manager: self.session_manager,
            tool_provider: self.tool_provider,
        };

        Router::new()
            // SSE endpoint for LLM clients to connect
            .route("/sse", get(sse_handler))
            .route("/sse/:session_id", get(sse_handler_with_session))
            
            // HTTP endpoints for LLM responses
            .route("/message", post(handle_message))
            .route("/tool_result", post(handle_tool_result))
            
            // Session management
            .route("/sessions", get(list_sessions))
            .route("/sessions", post(create_session))
            .route("/sessions/:id", axum::routing::delete(end_session))
            
            // Tool discovery
            .route("/tools", get(list_tools))
            
            // Health check
            .route("/health", get(health_check))
            
            .layer(CorsLayer::permissive())
            .with_state(Arc::new(state))
    }
}

#[derive(Clone)]
struct AppState {
    session_manager: Arc<SessionManager>,
    tool_provider: Arc<ToolProvider>,
}

/// SSE handler - LLM clients connect here to receive messages
async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (session_id, rx) = state.session_manager.create_session(None);
    info!("New SSE connection established: {}", session_id);
    
    sse_stream(session_id, rx, state)
}

/// SSE handler with specific session ID
async fn sse_handler_with_session(
    Path(session_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (_, rx) = state.session_manager.create_session(Some(session_id.clone()));
    info!("SSE connection with session ID: {}", session_id);
    
    sse_stream(session_id, rx, state)
}

/// Create SSE stream
fn sse_stream(
    session_id: SessionId,
    rx: mpsc::UnboundedReceiver<McpMessage>,
    state: Arc<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Send initial tools list before moving session_id
    if let Some(session) = state.session_manager.get_session(&session_id) {
        let tools_message = McpMessage::ToolsList {
            tools: state.tool_provider.list_tools(),
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
    State(state): State<Arc<AppState>>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    let session_id = &request.session_id;
    
    // Update session activity
    if let Err(e) = state.session_manager.update_session(session_id, |session| {
        session.update_activity();
    }) {
        return (StatusCode::NOT_FOUND, Json(json_error("Session not found", Some(e))));
    }
    
    // Process message
    match request.message {
        McpMessage::Initialize { client_info, .. } => {
            state.session_manager.update_session(session_id, |session| {
                session.set_client_info(client_info);
            }).ok();
            
            // Send initialized response
            if let Some(session) = state.session_manager.get_session(session_id) {
                let _ = session.send_message(McpMessage::Initialized);
            }
            
            (StatusCode::OK, Json(json_success("Initialized")))
        }
        
        McpMessage::Ping => {
            if let Some(session) = state.session_manager.get_session(session_id) {
                let _ = session.send_message(McpMessage::Pong);
            }
            (StatusCode::OK, Json(json_success("Pong")))
        }
        
        McpMessage::ListTools => {
            if let Some(session) = state.session_manager.get_session(session_id) {
                let tools_message = McpMessage::ToolsList {
                    tools: state.tool_provider.list_tools(),
                };
                let _ = session.send_message(tools_message);
            }
            (StatusCode::OK, Json(json_success("Tools list sent")))
        }
        
        McpMessage::ToolCall { id, tool, arguments } => {
            // Execute tool asynchronously
            let state_clone = state.clone();
            let session_id_clone = session_id.clone();
            let tool_id = id.clone();
            
            tokio::spawn(async move {
                execute_tool(state_clone, session_id_clone, tool_id, tool, arguments).await;
            });
            
            (StatusCode::ACCEPTED, Json(json_success("Tool execution started")))
        }
        
        McpMessage::Response { .. } | McpMessage::StreamingResponse { .. } => {
            // Forward response to connected IDE clients
            // This would integrate with chat-assistant plugin
            forward_to_ide(state.clone(), request.message).await;
            (StatusCode::OK, Json(json_success("Response forwarded")))
        }
        
        _ => {
            (StatusCode::OK, Json(json_success("Message processed")))
        }
    }
}

/// Execute a tool and send result back to LLM
async fn execute_tool(
    state: Arc<AppState>,
    session_id: String,
    tool_id: String,
    tool_name: String,
    arguments: serde_json::Value,
) {
    let result = if let Some(tool) = state.tool_provider.get_tool(&tool_name) {
        match tool.execute(arguments).await {
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
    
    if let Some(session) = state.session_manager.get_session(&session_id) {
        let _ = session.send_message(result);
    }
}

/// Handle tool execution results from LLM
async fn handle_tool_result(
    State(state): State<Arc<AppState>>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    // Forward tool results to IDE
    forward_to_ide(state, request.message).await;
    (StatusCode::OK, Json(json_success("Tool result received")))
}

/// Forward messages to connected IDE clients
async fn forward_to_ide(_state: Arc<AppState>, message: McpMessage) {
    // This will integrate with the chat-assistant plugin
    // through the existing WebSocket infrastructure
    info!("Forwarding to IDE: {:?}", message);
}

/// List active sessions
async fn list_sessions(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let sessions = state.session_manager.list_sessions();
    Json(sessions)
}

/// Create a new session
async fn create_session(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let (session_id, _) = state.session_manager.create_session(None);
    Json(serde_json::json!({
        "session_id": session_id
    }))
}

/// End a session
async fn end_session(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if state.session_manager.remove_session(&id).is_some() {
        (StatusCode::OK, Json(json_success("Session ended")))
    } else {
        (StatusCode::NOT_FOUND, Json(json_error("Session not found", None::<String>)))
    }
}

/// List available tools
async fn list_tools(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Json(state.tool_provider.list_tools())
}

/// Health check
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "mcp-server"
    }))
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