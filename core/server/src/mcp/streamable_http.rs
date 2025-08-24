use axum::{
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response, sse::{Event, KeepAlive, Sse}},
    Json,
};
use futures_util::stream::StreamExt;
use serde_json::{json, Value};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::websocket::WebSocketState;
use super::{
    jsonrpc::{JsonRpcRequest, JsonRpcResponse, METHOD_NOT_FOUND},
    session::SessionManager,
};

/// Handle streamable-http transport endpoint
/// Supports both GET (for SSE stream) and POST (for JSON-RPC requests)
pub async fn handle_streamable_http(
    method: Method,
    headers: HeaderMap,
    ws_state: Arc<WebSocketState>,
    session_manager: Arc<SessionManager>,
    body: Option<String>,
) -> Response {
    // Validate Origin header per spec
    if let Some(origin) = headers.get("origin") {
        let origin_str = origin.to_str().unwrap_or("");
        // Only allow localhost origins
        if !origin_str.starts_with("http://localhost") && 
           !origin_str.starts_with("http://127.0.0.1") &&
           !origin_str.starts_with("https://localhost") &&
           !origin_str.starts_with("https://127.0.0.1") {
            return (StatusCode::FORBIDDEN, "Invalid origin").into_response();
        }
    }

    match method {
        Method::GET => handle_get(headers, ws_state, session_manager).await,
        Method::POST => handle_post(headers, ws_state, session_manager, body.unwrap_or_default()).await,
        _ => (StatusCode::METHOD_NOT_ALLOWED, "Only GET and POST allowed").into_response(),
    }
}

/// Handle GET request - establish SSE stream
async fn handle_get(
    headers: HeaderMap,
    ws_state: Arc<WebSocketState>,
    session_manager: Arc<SessionManager>,
) -> Response {
    info!("=== MCP GET Request ===");
    
    // Log all headers
    for (name, value) in headers.iter() {
        info!("  Header: {} = {:?}", name, value.to_str().unwrap_or("(invalid)"));
    }
    
    // Check Accept header
    let accept = headers.get("accept").and_then(|v| v.to_str().ok()).unwrap_or("");
    info!("  Accept header value: '{}'", accept);
    
    if !accept.contains("text/event-stream") {
        error!("  REJECTED: Accept header must include text/event-stream");
        return (StatusCode::NOT_ACCEPTABLE, "Accept header must include text/event-stream").into_response();
    }

    // Check for existing session ID (for reconnection)
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    info!("  Received session ID from header: {:?}", session_id);

    // Generate temporary session if none provided
    let session_id = session_id.unwrap_or_else(|| {
        let temp_id = format!("temp-{}", Uuid::new_v4());
        info!("  Generated temporary session ID: {}", temp_id);
        temp_id
    });

    info!("  Establishing SSE stream for session: {}", session_id);

    // Create channel for this session
    let (tx, rx) = mpsc::unbounded_channel::<Value>();
    
    // Register the sender
    session_manager.register_sse_sender(session_id.clone(), tx.clone()).await;
    
    // Add to dashboard as MCP session
    ws_state.dashboard.add_mcp_session(session_id.clone()).await;

    // Send initial "endpoint-ready" message per streamable-http spec
    // This tells the client that the SSE connection is established and ready
    info!("  Sending endpoint-ready message");
    let ready_msg = json!({
        "type": "endpoint-ready"
    });
    if let Err(e) = tx.send(ready_msg) {
        error!("  Failed to send endpoint-ready: {}", e);
    }

    // Create SSE stream
    let stream = UnboundedReceiverStream::new(rx)
        .map(|message| -> Result<Event, Infallible> {
            let json_string = serde_json::to_string(&message).unwrap_or_else(|_| "{}".to_string());
            Ok(Event::default()
                .event("message")
                .data(json_string))
        });

    // Return SSE response
    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text(":"),
        )
        .into_response()
}

/// Handle POST request - process JSON-RPC
async fn handle_post(
    headers: HeaderMap,
    ws_state: Arc<WebSocketState>,
    session_manager: Arc<SessionManager>,
    body: String,
) -> Response {
    info!("=== MCP POST Request ===");
    
    // Log all headers
    for (name, value) in headers.iter() {
        info!("  Header: {} = {:?}", name, value.to_str().unwrap_or("(invalid)"));
    }
    
    // Check Accept header
    let accept = headers.get("accept").and_then(|v| v.to_str().ok()).unwrap_or("");
    info!("  Accept header value: '{}'", accept);
    
    if !accept.contains("application/json") && !accept.contains("text/event-stream") {
        error!("  REJECTED: Accept header must include application/json or text/event-stream");
        return Json(JsonRpcResponse::error(
            None,
            -32600,
            "Accept header must include application/json or text/event-stream".to_string(),
            None,
        )).into_response();
    }

    info!("  Request body: {}", body);

    // Parse JSON-RPC request
    let request: JsonRpcRequest = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(e) => {
            error!("  Failed to parse JSON-RPC: {}", e);
            return Json(JsonRpcResponse::error(
                None,
                -32700,
                "Parse error".to_string(),
                Some(json!({"error": e.to_string()})),
            )).into_response();
        }
    };

    info!("  Method: {}", request.method);
    info!("  Request ID: {:?}", request.id);
    info!("  Params: {:?}", request.params);

    // Get session ID from header
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    info!("  Session ID from header: {:?}", session_id);

    // Track the actual session ID to use for sending response
    let mut response_session_id = session_id.clone();

    // Handle the request based on method
    let response = match request.method.as_str() {
        "initialize" => {
            info!("  Processing initialize request");
            
            // Generate new session ID
            let new_session_id = format!("session-{}", Uuid::new_v4());
            info!("  Generated new session ID: {}", new_session_id);
            
            // If there was a temp session, update it
            if let Some(ref old_id) = session_id {
                info!("  Updating session ID from {} to {}", old_id, new_session_id);
                session_manager.update_session_id(old_id, new_session_id.clone()).await;
                // Use the new session ID for sending response
                response_session_id = Some(new_session_id.clone());
            } else {
                info!("  No previous session to update");
                response_session_id = Some(new_session_id.clone());
            }
            
            let params = request.params.unwrap_or(json!({}));
            let client_info = params.get("clientInfo").cloned().unwrap_or(json!({
                "name": "unknown",
                "version": "0.0.0"
            }));
            info!("  Client info: {:?}", client_info);
            
            // Send MCP connected status to the browser
            let connected_event = json!({
                "type": "llm_connected",
                "data": {
                    "session_id": &new_session_id,
                    "client_info": client_info.clone()
                }
            });
            
            let payload = serde_json::to_vec(&connected_event).unwrap_or_default();
            let packet = crate::packet::Packet {
                channel_id: 1050,
                packet_type: 2, // Status update type
                priority: crate::packet::Priority::High,
                payload: bytes::Bytes::from(payload),
            };
            
            ws_state.batcher.queue_packet(packet).await;
            info!("  MCP connected status sent to channel 1050");
            
            JsonRpcResponse::success(request.id, json!({
                "protocolVersion": "2025-06-18",
                "serverInfo": {
                    "name": "android-playground",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    },
                    "prompts": {
                        "listChanged": true
                    },
                    "resources": {
                        "subscribe": false,
                        "listChanged": false
                    },
                    "logging": {}
                },
                "sessionId": new_session_id
            }))
        },
        
        "tools/list" => {
            JsonRpcResponse::success(request.id, json!({
                "tools": get_available_tools(ws_state.clone()).await
            }))
        },
        
        "tools/call" => {
            let params = request.params.unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            
            info!("  Tool call: '{}' with args: {:?}", tool_name, arguments);
            
            // Handle built-in test tools directly
            let result = match tool_name {
                "ping" => {
                    let message = arguments.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("pong");
                    json!({
                        "content": [{
                            "type": "text",
                            "text": format!("pong: {}", message)
                        }]
                    })
                },
                
                "echo" => {
                    let data = arguments.get("data").cloned().unwrap_or(json!("(no data)"));
                    json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Echo: {}", serde_json::to_string_pretty(&data).unwrap_or_default())
                        }]
                    })
                },
                
                "get_status" => {
                    json!({
                        "content": [{
                            "type": "text",
                            "text": format!("MCP Server Status:\n- Session: {}\n- Active: true\n- Version: 0.1.0", 
                                          session_id.as_deref().unwrap_or("none"))
                        }]
                    })
                },
                
                "list_channels" => {
                    // Get channel manager info
                    let channels = ws_state.channel_manager.list_channels().await;
                    let channel_info = channels
                        .into_iter()
                        .map(|info| format!("  - Channel {}: {} ({})", info.id, info.name, info.owner))
                        .collect::<Vec<_>>()
                        .join("\n");
                    
                    json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Registered WebSocket Channels:\n{}", 
                                          if channel_info.is_empty() { "  (none)" } else { &channel_info })
                        }]
                    })
                },
                
                // UI Framework tools - forward to channel 1200
                "show_file" | "update_editor" | "show_terminal_output" => {
                    info!("  Forwarding UI tool '{}' to channel 1200", tool_name);
                    
                    let tool_call_event = json!({
                        "type": "tool_call",
                        "session_id": session_id.as_deref().unwrap_or("unknown"),
                        "tool": tool_name,
                        "arguments": arguments,
                    });
                    
                    let payload = serde_json::to_vec(&tool_call_event).unwrap_or_default();
                    let packet = crate::packet::Packet {
                        channel_id: 1200,
                        packet_type: 1,
                        priority: crate::packet::Priority::High,
                        payload: bytes::Bytes::from(payload),
                    };
                    
                    ws_state.batcher.queue_packet(packet).await;
                    info!("  Tool call queued for channel 1200");
                    
                    json!({
                        "content": [{
                            "type": "text",
                            "text": format!("UI tool '{}' forwarded to browser", tool_name)
                        }]
                    })
                },
                
                // Check if it's a dynamically registered tool
                _ => {
                    // Look up in registered tools
                    let registered_tools = ws_state.get_mcp_tools().await;
                    if let Some(tool) = registered_tools.iter().find(|t| t.name == tool_name) {
                        info!("  Forwarding registered tool '{}' to channel {}", tool_name, tool.handler_channel);
                        
                        let tool_call_event = json!({
                            "type": "tool_call",
                            "session_id": session_id.as_deref().unwrap_or("unknown"),
                            "tool": tool_name,
                            "arguments": arguments,
                        });
                        
                        let payload = serde_json::to_vec(&tool_call_event).unwrap_or_default();
                        let packet = crate::packet::Packet {
                            channel_id: tool.handler_channel,
                            packet_type: 1,
                            priority: crate::packet::Priority::High,
                            payload: bytes::Bytes::from(payload),
                        };
                        
                        ws_state.batcher.queue_packet(packet).await;
                        info!("  Tool call queued for channel {}", tool.handler_channel);
                        
                        json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Tool '{}' executed via channel {}", tool_name, tool.handler_channel)
                            }]
                        })
                    } else {
                        json!({
                            "isError": true,
                            "content": [{
                                "type": "text",
                                "text": format!("Unknown tool: {}", tool_name)
                            }]
                        })
                    }
                }
            };
            
            JsonRpcResponse::success(request.id, result)
        },
        
        "prompts/list" => {
            JsonRpcResponse::success(request.id, json!({
                "prompts": []
            }))
        },
        
        "resources/list" => {
            JsonRpcResponse::success(request.id, json!({
                "resources": []
            }))
        },
        
        "initialized" => {
            // This is a notification, not a request - no response needed
            // Just acknowledge we received it
            info!("Client initialized notification received");
            JsonRpcResponse::success(request.id, json!({}))
        },
        
        "ping" => {
            JsonRpcResponse::success(request.id, json!({}))
        },
        
        _ => {
            JsonRpcResponse::error(
                request.id,
                METHOD_NOT_FOUND,
                format!("Method '{}' not found", request.method),
                None
            )
        }
    };

    // For initialize request, ALWAYS return JSON with session ID header (per spec)
    if request.method == "initialize" {
        let mut headers = axum::http::HeaderMap::new();
        if let Some(session_id) = response.result.as_ref()
            .and_then(|r| r.get("sessionId"))
            .and_then(|s| s.as_str()) {
            info!("  Adding Mcp-Session-Id header: {}", session_id);
            if let Ok(header_value) = axum::http::HeaderValue::from_str(session_id) {
                headers.insert(
                    axum::http::HeaderName::from_static("mcp-session-id"),
                    header_value,
                );
            }
        }
        info!("  Returning initialize response as JSON with headers");
        info!("  Response: {:?}", response);
        return (headers, Json(response)).into_response();
    }
    
    // For notifications (no id field), return 202 Accepted
    if response.id.is_none() {
        info!("  Notification received, returning 202 Accepted");
        return StatusCode::ACCEPTED.into_response();
    }
    
    // For other requests, check if client wants SSE response
    if accept.contains("text/event-stream") {
        let target_session = if let Some(id) = response_session_id {
            Some(id)
        } else {
            session_manager.get_last_sse_session().await
        };
        
        if let Some(session_id) = target_session {
            // Send response via SSE stream
            if let Err(e) = session_manager.send_to_sse(&session_id, json!(response)).await {
                error!("Failed to send via SSE: {}", e);
                // Fall back to JSON response
                return Json(response).into_response();
            }
            // Return empty SSE stream (response was sent via existing SSE connection)
            let stream = futures_util::stream::empty::<Result<Event, Infallible>>();
            return Sse::new(stream).into_response();
        }
        warn!("No SSE session available, falling back to JSON");
    }

    // Default: Return JSON response directly
    Json(response).into_response()
}

/// Get list of available tools (both built-in and dynamically registered)
async fn get_available_tools(ws_state: Arc<WebSocketState>) -> Vec<Value> {
    let mut tools = vec![
        // Test/diagnostic tools
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
            "name": "list_channels",
            "description": "List all registered WebSocket channels",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        // UI Framework tools (forward to channel 1200)
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
    ];
    
    // Add dynamically registered tools from WebSocketState
    let registered_tools = ws_state.get_mcp_tools().await;
    for tool in registered_tools {
        tools.push(json!({
            "name": tool.name,
            "description": tool.description,
            "inputSchema": tool.input_schema
        }));
    }
    
    tools
}