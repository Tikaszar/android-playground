use axum::{
    extract::{Path, State, Request},
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, post},
    Json, Router,
};
use std::sync::Arc;
use tracing::{info, debug, error};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::websocket::WebSocketState;
use super::{
    jsonrpc::{JsonRpcRequest, JsonRpcResponse},
    session::{SessionManager, SessionInfo},
    streamable_http,
};

/// MCP Server state
struct McpState {
    session_manager: Arc<SessionManager>,
}

/// MCP Server implementation
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
            // Main streamable-http endpoint - handles both GET and POST
            .route("/", any(handle_streamable))
            
            // Session management endpoints
            .route("/sessions", get(list_sessions))
            .route("/sessions", post(create_session))
            .route("/sessions/{id}", axum::routing::delete(end_session))
            
            // Tool discovery
            .route("/tools", get(list_tools))
            
            // Health check
            .route("/health", get(health_check))
            
            // Dynamic client registration (stub for Claude Code)
            .route("/register", post(register_client))
            .route("/clients/register", post(register_client))
            
            .layer(axum::extract::Extension(mcp_state))
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Main streamable-http handler
async fn handle_streamable(
    State(ws_state): State<Arc<WebSocketState>>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
    req: Request,
) -> impl IntoResponse {
    let method = req.method().clone();
    let headers = req.headers().clone();
    
    // Extract body for POST requests
    let body = if method == axum::http::Method::POST {
        let (parts, body) = req.into_parts();
        let bytes = match axum::body::to_bytes(body, usize::MAX).await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to read body: {}", e);
                return (StatusCode::BAD_REQUEST, "Failed to read body").into_response();
            }
        };
        Some(String::from_utf8_lossy(&bytes).to_string())
    } else {
        None
    };
    
    streamable_http::handle_streamable_http(
        method,
        headers,
        ws_state,
        mcp_state.session_manager.clone(),
        body,
    ).await
}

/// List active sessions
async fn list_sessions(
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> Json<Value> {
    let sessions = mcp_state.session_manager.list_sessions();
    Json(json!({
        "sessions": sessions
    }))
}

/// Create a new session
async fn create_session(
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let session_id = body.get("sessionId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let (session_id, _rx) = mcp_state.session_manager.create_session(session_id);
    
    Json(json!({
        "sessionId": session_id
    }))
}

/// End a session
async fn end_session(
    Path(id): Path<String>,
    axum::extract::Extension(mcp_state): axum::extract::Extension<Arc<McpState>>,
) -> StatusCode {
    if mcp_state.session_manager.remove_session(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// List available tools
async fn list_tools() -> Json<Value> {
    // Diagnostic/test tools for debugging MCP connection
    let test_tools = vec![
        json!({
            "name": "ping",
            "description": "Test MCP connection - responds with pong",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Optional message to echo back"
                    }
                }
            }
        }),
        json!({
            "name": "echo",
            "description": "Echo back any input for testing",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "data": {
                        "type": "any",
                        "description": "Any data to echo back"
                    }
                }
            }
        }),
        json!({
            "name": "get_status",
            "description": "Get current MCP server status",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "test_ui_framework",
            "description": "Test if UI Framework Plugin is receiving messages",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "test_message": {
                        "type": "string",
                        "description": "Test message to send to UI Framework"
                    }
                }
            }
        }),
        json!({
            "name": "list_channels",
            "description": "List all registered WebSocket channels",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
    ];
    
    // UI Framework tools (these forward to channel 1200)
    let ui_tools = vec![
        json!({
            "name": "show_file",
            "description": "Display file content in editor bubble",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }
        }),
        json!({
            "name": "update_editor",
            "description": "Update current editor content",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "content": { "type": "string" }
                },
                "required": ["content"]
            }
        }),
        json!({
            "name": "show_terminal_output",
            "description": "Display terminal output",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "output": { "type": "string" }
                },
                "required": ["output"]
            }
        }),
    ];
    
    let mut all_tools = test_tools;
    all_tools.extend(ui_tools);
    
    Json(json!({
        "tools": all_tools
    }))
}

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "mcp-server"
    }))
}

/// Dynamic client registration stub
async fn register_client(Json(body): Json<Value>) -> Json<Value> {
    info!("Client registration request: {:?}", body);
    
    // Return a minimal successful registration response
    // This is just to satisfy Claude Code's expectation
    Json(json!({
        "client_id": Uuid::new_v4().to_string(),
        "client_name": body.get("client_name").and_then(|v| v.as_str()).unwrap_or("claude-code"),
        "registration_access_token": "not-required",
        "grant_types": ["implicit"],
        "response_types": ["token"]
    }))
}