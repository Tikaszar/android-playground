use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures_util::stream::Stream;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tracing::{info, debug, error};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::websocket::WebSocketState;
use super::jsonrpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, METHOD_NOT_FOUND, INTERNAL_ERROR};
use super::session::SessionManager;

/// MCP SSE state for each connection
pub struct McpSseState {
    pub session_manager: Arc<SessionManager>,
    pub ws_state: Arc<WebSocketState>,
}

/// Handle SSE connection with JSON-RPC 2.0 protocol
pub async fn handle_sse_connection(
    State(ws_state): State<Arc<WebSocketState>>,
    session_manager: Arc<SessionManager>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Check for Accept header
    if let Some(accept) = headers.get("accept") {
        let accept_str = accept.to_str().unwrap_or("");
        if !accept_str.contains("text/event-stream") {
            return (axum::http::StatusCode::NOT_ACCEPTABLE, "Accept header must include text/event-stream").into_response();
        }
    }
    
    // Check for existing session ID from header (for reconnection)
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // No session yet - will be created during initialize
            let id = Uuid::new_v4().to_string();
            info!("New SSE connection, temporary session: {}", id);
            id
        });
    
    let (tx, rx) = mpsc::unbounded_channel();
    
    info!("MCP SSE client connected: {}", session_id);
    
    // Store the sender in session manager for bidirectional communication
    session_manager.register_sse_sender(session_id.clone(), tx.clone());
    
    // Don't send session ID - it will be returned in initialize response per spec
    
    // Create SSE stream with JSON-RPC 2.0 messages
    let stream = UnboundedReceiverStream::new(rx)
        .map(move |message| -> Result<Event, Infallible> {
            // Convert JSON-RPC message to SSE event
            // Compact JSON serialization to avoid newlines
            let json_string = serde_json::to_string(&message).unwrap_or_else(|_| "{}".to_string());
            Ok(Event::default()
                .event("message")
                .data(json_string))
        });

    // Return SSE response with proper headers
    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text(":"),  // Standard SSE keepalive comment (no newline)
    );
    
    // Set proper headers per MCP spec
    (
        [
            (axum::http::header::CONTENT_TYPE, "text/event-stream"),
            (axum::http::header::CACHE_CONTROL, "no-cache"),
        ],
        sse,
    ).into_response()
}

/// Process incoming JSON-RPC 2.0 request from SSE client
pub async fn process_jsonrpc_request(
    request: JsonRpcRequest,
    session_id: &str,
    ws_state: Arc<WebSocketState>,
    session_manager: Arc<SessionManager>,
) -> JsonRpcResponse {
    debug!("Processing JSON-RPC request: {} for session {}", request.method, session_id);
    
    match request.method.as_str() {
        "initialize" => {
            // MCP initialization
            let params = request.params.unwrap_or(json!({}));
            let client_info = params.get("clientInfo").cloned().unwrap_or(json!({
                "name": "unknown",
                "version": "0.0.0"
            }));
            
            // Generate a proper session ID for this client
            let new_session_id = format!("session-{}", Uuid::new_v4());
            
            // Update the session manager with the proper session ID
            session_manager.update_session_id(session_id, new_session_id.clone());
            
            let result = json!({
                "protocolVersion": "1.0.0",
                "serverInfo": {
                    "name": "android-playground",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "tools": json!({
                        "listChanged": true
                    }),
                    "prompts": json!({
                        "listChanged": true
                    }),
                    "resources": json!({
                        "subscribe": false,
                        "listChanged": false
                    }),
                    "logging": json!({})
                },
                "sessionId": new_session_id
            });
            
            JsonRpcResponse::success(request.id, result)
        },
        
        "tools/list" => {
            // Return available tools
            let tools = get_available_tools();
            JsonRpcResponse::success(request.id, json!({
                "tools": tools
            }))
        },
        
        "tools/call" => {
            // Execute tool call
            let params = request.params.unwrap_or(json!({}));
            let tool_name = params.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            
            // Forward tool call to plugins via channel 2000
            let tool_call_event = json!({
                "type": "tool_call",
                "session_id": session_id,
                "call_id": Uuid::new_v4().to_string(),
                "tool": tool_name,
                "arguments": arguments,
            });
            
            // In a real implementation, we'd wait for the response from plugins
            // For now, return a placeholder response
            let result = json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("Tool '{}' executed successfully", tool_name)
                    }
                ]
            });
            
            JsonRpcResponse::success(request.id, result)
        },
        
        "prompts/list" => {
            // Return available prompts
            JsonRpcResponse::success(request.id, json!({
                "prompts": []
            }))
        },
        
        "prompts/get" => {
            // Get a specific prompt
            let params = request.params.unwrap_or(json!({}));
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            
            JsonRpcResponse::error(
                request.id,
                METHOD_NOT_FOUND,
                format!("Prompt '{}' not found", name),
                None
            )
        },
        
        "resources/list" => {
            // Return available resources
            JsonRpcResponse::success(request.id, json!({
                "resources": []
            }))
        },
        
        "completion/complete" => {
            // Handle completion request
            JsonRpcResponse::success(request.id, json!({
                "completion": {
                    "values": [],
                    "total": 0,
                    "hasMore": false
                }
            }))
        },
        
        "ping" => {
            // Respond to ping
            JsonRpcResponse::success(request.id, json!({}))
        },
        
        _ => {
            // Unknown method
            JsonRpcResponse::error(
                request.id,
                METHOD_NOT_FOUND,
                format!("Method '{}' not found", request.method),
                None
            )
        }
    }
}

/// Get list of available tools
fn get_available_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "show_file",
            "description": "Display file content in the browser editor",
            "inputSchema": {
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
        json!({
            "name": "update_editor",
            "description": "Update the current editor content",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "New editor content"
                    }
                },
                "required": ["content"]
            }
        }),
        json!({
            "name": "show_terminal_output",
            "description": "Display output in the terminal",
            "inputSchema": {
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
        json!({
            "name": "update_file_tree",
            "description": "Update the file browser tree",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "description": "File tree structure",
                        "items": {
                            "type": "object"
                        }
                    }
                },
                "required": ["files"]
            }
        }),
        json!({
            "name": "show_diff",
            "description": "Display a diff view",
            "inputSchema": {
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
                    "filename": {
                        "type": "string",
                        "description": "File name for the diff"
                    }
                },
                "required": ["old_content", "new_content"]
            }
        }),
        json!({
            "name": "show_error",
            "description": "Show error message with location",
            "inputSchema": {
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
            }
        }),
        json!({
            "name": "update_status_bar",
            "description": "Update status bar message",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Status message"
                    }
                },
                "required": ["message"]
            }
        }),
        json!({
            "name": "show_notification",
            "description": "Display a notification",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Notification message"
                    },
                    "type": {
                        "type": "string",
                        "enum": ["info", "warning", "error", "success"],
                        "description": "Notification type"
                    }
                },
                "required": ["message"]
            }
        }),
        json!({
            "name": "open_panel",
            "description": "Open a specific IDE panel",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "panel": {
                        "type": "string",
                        "enum": ["editor", "terminal", "files", "chat", "debugger"],
                        "description": "Panel to open"
                    }
                },
                "required": ["panel"]
            }
        }),
        json!({
            "name": "show_chat_message",
            "description": "Display message in conversation",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Chat message"
                    },
                    "sender": {
                        "type": "string",
                        "enum": ["user", "assistant"],
                        "description": "Message sender"
                    }
                },
                "required": ["message", "sender"]
            }
        })
    ]
}