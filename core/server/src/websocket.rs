use crate::{
    channel::ChannelManager,
    packet::{Packet, Priority, ControlMessageType},
    batcher::FrameBatcher,
    dashboard::Dashboard,
};
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State},
    response::Response,
};
use std::sync::Arc;
use std::collections::HashMap;
use playground_core_types::{Handle, handle, Shared, shared};
use futures_util::{StreamExt, SinkExt};
use bytes::{Bytes, BytesMut, BufMut};
// Dashboard logging is used instead of tracing
use tokio::time;
use serde_json::Value;

/// MCP Tool definition
#[derive(Clone, Debug)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub handler_channel: u16, // Channel to forward tool calls to
}

pub struct WebSocketState {
    pub channel_manager: Handle<ChannelManager>,
    pub batcher: Handle<FrameBatcher>,
    pub connections: Shared<Vec<Shared<Option<WebSocketConnection>>>>,
    pub mcp_tools: Shared<HashMap<String, McpTool>>, // Dynamic MCP tool registry
    pub dashboard: Handle<Dashboard>,
}

pub struct WebSocketConnection {
    id: usize,
    sender: futures_util::stream::SplitSink<WebSocket, Message>,
}

impl WebSocketConnection {
    pub async fn send(&mut self, message: Message) -> Result<(), String> {
        use futures_util::SinkExt;
        self.sender.send(message).await.map_err(|e| e.to_string())
    }
}

impl WebSocketState {
    pub fn new() -> Self {
        let dashboard = handle(Dashboard::new());
        
        Self {
            channel_manager: handle(ChannelManager::new()),
            batcher: handle(FrameBatcher::new(2000, 60)), // 60fps, support up to 2000 channels
            connections: shared(Vec::new()),
            mcp_tools: shared(HashMap::new()),
            dashboard,
        }
    }
    
    pub fn new_with_dashboard(dashboard: Handle<Dashboard>) -> Self {
        Self {
            channel_manager: handle(ChannelManager::new()),
            batcher: handle(FrameBatcher::new(2000, 60)), // 60fps, support up to 2000 channels
            connections: shared(Vec::new()),
            mcp_tools: shared(HashMap::new()),
            dashboard,
        }
    }
    
    /// Register an MCP tool that can be called by LLMs
    pub async fn register_mcp_tool(&self, tool: McpTool) {
        let mut tools = self.mcp_tools.write().await;
        self.dashboard.log(
            crate::dashboard::LogLevel::Info,
            format!("Registering MCP tool: {} (handler: channel {})", tool.name, tool.handler_channel),
            None
        ).await;
        tools.insert(tool.name.clone(), tool);
    }
    
    /// Unregister an MCP tool
    pub async fn unregister_mcp_tool(&self, name: &str) {
        let mut tools = self.mcp_tools.write().await;
        if tools.remove(name).is_some() {
            self.dashboard.log(
                crate::dashboard::LogLevel::Info,
                format!("Unregistered MCP tool: {}", name),
                None
            ).await;
        }
    }
    
    /// Get all registered MCP tools
    pub async fn get_mcp_tools(&self) -> Vec<McpTool> {
        let tools = self.mcp_tools.read().await;
        tools.values().cloned().collect()
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Handle<WebSocketState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: Handle<WebSocketState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Get client IP (placeholder - in real implementation, extract from request headers)
    let client_ip = "127.0.0.1".to_string();
    
    let connection_id = {
        let mut connections = state.connections.write().await;
        let id = connections.len();
        connections.push(shared(Some(WebSocketConnection {
            id,
            sender,
        })));
        id
    };
    
    // Add client to dashboard
    state.dashboard.add_client(connection_id, client_ip.clone()).await;
    
    // Connection logging handled by dashboard.add_client
    
    let state_clone = state.clone();
    let send_task = tokio::spawn(async move {
        let mut interval = time::interval(state_clone.batcher.frame_duration());
        
        loop {
            interval.tick().await;
            
            // Only get batches if this is connection 0 (to avoid multiple connections consuming the same packets)
            // This is a temporary fix - we should implement proper broadcast
            if connection_id == 0 {
                let batches = state_clone.batcher.get_all_batches().await;
                if !batches.is_empty() {
                    state_clone.dashboard.log(
                        crate::dashboard::LogLevel::Debug,
                        format!("Broadcasting {} batches to all clients", batches.len()),
                        None
                    ).await;
                    
                    // Send to ALL connections, not just this one
                    let connections = state_clone.connections.read().await;
                    
                    state_clone.dashboard.log(
                        crate::dashboard::LogLevel::Debug,
                        format!("Total connections: {}", connections.len()),
                        None
                    ).await;
                    
                    for (conn_id, conn_lock) in connections.iter().enumerate() {
                        let mut conn = conn_lock.write().await;
                        if let Some(connection) = conn.as_mut() {
                            for (_channel_id, batch) in &batches {
                                let batch_len = batch.len() as u64;
                                
                                // Update dashboard with sent message
                                state_clone.dashboard.update_client_activity(conn_id, false, batch_len).await;
                                
                                if let Err(e) = connection.sender.send(Message::Binary(batch.clone())).await {
                                    state_clone.dashboard.log_error(
                                        format!("Failed to send batch to client {}: {}", conn_id, e), 
                                        Some(conn_id)
                                    ).await;
                                    // Can't set to None here while borrowed
                                    break;
                                }
                            }
                        }
                    }
                }
            } else {
                // Other connections just wait
                continue;
            }
        }
    });
    
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                let data_len = data.len() as u64;
                // Update dashboard with received message
                state.dashboard.update_client_activity(connection_id, true, data_len).await;
                
                if let Err(e) = handle_message(Bytes::from(data), &state).await {
                    let error_msg = format!("Error handling message: {}", e);
                    // Error logged to dashboard below
                    state.dashboard.log_error(error_msg, Some(connection_id)).await;
                }
            }
            Ok(Message::Close(_)) => {
                // Disconnection handled by dashboard.remove_client
                break;
            }
            Err(e) => {
                state.dashboard.log_error(format!("WebSocket error: {}", e), Some(connection_id)).await;
                break;
            }
            _ => {}
        }
    }
    
    send_task.abort();
    
    // Mark client as disconnected in dashboard
    state.dashboard.remove_client(connection_id).await;
    
    let connections = state.connections.write().await;
    if connection_id < connections.len() {
        *connections[connection_id].write().await = None;
    }
    
    // Disconnection logged by dashboard.remove_client
}

async fn handle_message(data: Bytes, state: &WebSocketState) -> anyhow::Result<()> {
    let packet = Packet::deserialize(data)?;
    
    if packet.channel_id == 0 {
        handle_control_message(packet, state).await?;
    } else {
        // Instead of queuing, broadcast immediately to all OTHER clients
        // This is a temporary fix for the broadcast issue
        let connections = state.connections.read().await;
        let packet_bytes = packet.serialize();
        
        state.dashboard.log(
            crate::dashboard::LogLevel::Debug,
            format!("Broadcasting packet (channel {}, type {}) to {} clients", 
                packet.channel_id, packet.packet_type, connections.len()),
            None
        ).await;
        
        for (conn_id, conn_lock) in connections.iter().enumerate() {
            // Send to ALL clients, including Client #0
            // We want everyone to receive the broadcast
            
            let mut conn = conn_lock.write().await;
            if let Some(connection) = conn.as_mut() {
                if let Err(e) = connection.sender.send(Message::Binary(packet_bytes.clone())).await {
                    state.dashboard.log(
                        crate::dashboard::LogLevel::Error,
                        format!("Failed to broadcast to client {}: {}", conn_id, e),
                        Some(conn_id)
                    ).await;
                } else {
                    state.dashboard.log(
                        crate::dashboard::LogLevel::Debug,
                        format!("Sent packet to client {}", conn_id),
                        Some(conn_id)
                    ).await;
                }
            }
        }
        
        // Also queue it for batching (in case we want to use that later)
        state.batcher.queue_packet(packet).await;
    }
    
    Ok(())
}

async fn handle_control_message(packet: Packet, state: &WebSocketState) -> anyhow::Result<()> {
    // Check for browser log messages (packet_type 200)
    if packet.packet_type == 200 {
        // Browser log message
        if let Ok(log_data) = serde_json::from_slice::<serde_json::Value>(&packet.payload) {
            if let (Some(level), Some(message)) = (
                log_data.get("level").and_then(|v| v.as_str()),
                log_data.get("message").and_then(|v| v.as_str()),
            ) {
                let log_level = match level {
                    "error" => crate::dashboard::LogLevel::Error,
                    "warning" | "warn" => crate::dashboard::LogLevel::Warning,
                    "info" => crate::dashboard::LogLevel::Info,
                    _ => crate::dashboard::LogLevel::Debug,
                };
                
                state.dashboard.log(
                    log_level,
                    format!("[Browser] {}", message),
                    None
                ).await;
            }
        }
        return Ok(());
    }
    
    // Check for MCP tool registration messages (packet_type 100 and 101)
    if packet.packet_type == 100 {
        // Register MCP tool
        let registration: serde_json::Value = serde_json::from_slice(&packet.payload)?;
        
        if let (Some(name), Some(description), Some(input_schema), Some(handler_channel)) = (
            registration.get("name").and_then(|v| v.as_str()),
            registration.get("description").and_then(|v| v.as_str()),
            registration.get("input_schema"),
            registration.get("handler_channel").and_then(|v| v.as_u64()),
        ) {
            let tool = McpTool {
                name: name.to_string(),
                description: description.to_string(),
                input_schema: input_schema.clone(),
                handler_channel: handler_channel as u16,
            };
            
            state.register_mcp_tool(tool).await;
            state.dashboard.log(
                crate::dashboard::LogLevel::Info,
                format!("Registered MCP tool '{}' for channel {}", name, handler_channel),
                None
            ).await;
        }
        return Ok(());
    } else if packet.packet_type == 101 {
        // Unregister MCP tool
        let unregistration: serde_json::Value = serde_json::from_slice(&packet.payload)?;
        
        if let Some(name) = unregistration.get("name").and_then(|v| v.as_str()) {
            state.unregister_mcp_tool(name).await;
            state.dashboard.log(
                crate::dashboard::LogLevel::Info,
                format!("Unregistered MCP tool '{}'", name),
                None
            ).await;
        }
        return Ok(());
    }
    
    let msg_type = ControlMessageType::try_from(packet.packet_type)?;
    
    match msg_type {
        ControlMessageType::RegisterSystem => {
            let name = String::from_utf8(packet.payload.to_vec())?;
            let channel_id = name.split(':')
                .nth(1)
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);
            
            let name = name.split(':').next().unwrap_or(&name).to_string();
            
            match state.channel_manager.register_system(name.clone(), channel_id).await {
                Ok(id) => {
                    let response = create_register_response(id);
                    state.batcher.queue_packet(response).await;
                    state.dashboard.log(
                        crate::dashboard::LogLevel::Info,
                        format!("Registered system '{}' on channel {}", name, id),
                        None
                    ).await;
                }
                Err(e) => {
                    let error = create_error_response(format!("Failed to register system: {}", e));
                    state.batcher.queue_packet(error).await;
                }
            }
        }
        ControlMessageType::RegisterPlugin => {
            let name = String::from_utf8(packet.payload.to_vec())?;
            
            match state.channel_manager.register_plugin(name.clone()).await {
                Ok(id) => {
                    let response = create_register_response(id);
                    state.batcher.queue_packet(response).await;
                    state.dashboard.log(
                        crate::dashboard::LogLevel::Info,
                        format!("Registered plugin '{}' on channel {}", name, id),
                        None
                    ).await;
                }
                Err(e) => {
                    let error = create_error_response(format!("Failed to register plugin: {}", e));
                    state.batcher.queue_packet(error).await;
                }
            }
        }
        ControlMessageType::QueryChannel => {
            let name = String::from_utf8(packet.payload.to_vec())?;
            
            if let Some(info) = state.channel_manager.get_channel_by_name(&name).await {
                let response = create_query_response(info.id, info.name);
                state.batcher.queue_packet(response).await;
            } else {
                let error = create_error_response(format!("Channel '{}' not found", name));
                state.batcher.queue_packet(error).await;
            }
        }
        ControlMessageType::ListChannels => {
            let channels = state.channel_manager.list_channels().await;
            let response = create_list_response(channels);
            state.batcher.queue_packet(response).await;
        }
        _ => {
            // Ignoring unhandled control message type
        }
    }
    
    Ok(())
}

fn create_register_response(channel_id: u16) -> Packet {
    let mut payload = BytesMut::new();
    payload.put_u16(channel_id);
    
    Packet::new(
        0,
        ControlMessageType::RegisterResponse as u16,
        Priority::High,
        payload.freeze(),
    )
}

fn create_query_response(channel_id: u16, name: String) -> Packet {
    let mut payload = BytesMut::new();
    payload.put_u16(channel_id);
    payload.put_u16(name.len() as u16);
    payload.put(name.as_bytes());
    
    Packet::new(
        0,
        ControlMessageType::QueryResponse as u16,
        Priority::High,
        payload.freeze(),
    )
}

fn create_list_response(channels: Vec<crate::channel::ChannelInfo>) -> Packet {
    let mut payload = BytesMut::new();
    payload.put_u16(channels.len() as u16);
    
    for channel in channels {
        payload.put_u16(channel.id);
        payload.put_u16(channel.name.len() as u16);
        payload.put(channel.name.as_bytes());
        payload.put_u16(channel.owner.len() as u16);
        payload.put(channel.owner.as_bytes());
    }
    
    Packet::new(
        0,
        ControlMessageType::ListResponse as u16,
        Priority::High,
        payload.freeze(),
    )
}

fn create_error_response(error: String) -> Packet {
    Packet::new(
        0,
        ControlMessageType::Error as u16,
        Priority::Critical,
        Bytes::from(error.into_bytes()),
    )
}