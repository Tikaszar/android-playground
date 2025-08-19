use crate::{
    channel::ChannelManager,
    packet::{Packet, Priority, ControlMessage, ControlMessageType},
    batcher::FrameBatcher,
};
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State},
    response::Response,
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use futures_util::{StreamExt, SinkExt};
use bytes::{Bytes, BytesMut, BufMut};
use tracing::{info, error, debug};
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
    pub channel_manager: Arc<RwLock<ChannelManager>>,
    pub batcher: Arc<FrameBatcher>,
    pub connections: Arc<RwLock<Vec<Arc<RwLock<Option<WebSocketConnection>>>>>>,
    pub mcp_tools: Arc<RwLock<HashMap<String, McpTool>>>, // Dynamic MCP tool registry
}

struct WebSocketConnection {
    id: usize,
    sender: futures_util::stream::SplitSink<WebSocket, Message>,
}

impl WebSocketState {
    pub fn new() -> Self {
        Self {
            channel_manager: Arc::new(RwLock::new(ChannelManager::new())),
            batcher: Arc::new(FrameBatcher::new(2000, 60)), // 60fps, support up to 2000 channels
            connections: Arc::new(RwLock::new(Vec::new())),
            mcp_tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register an MCP tool that can be called by LLMs
    pub async fn register_mcp_tool(&self, tool: McpTool) {
        let mut tools = self.mcp_tools.write().await;
        info!("Registering MCP tool: {} (handler: channel {})", tool.name, tool.handler_channel);
        tools.insert(tool.name.clone(), tool);
    }
    
    /// Unregister an MCP tool
    pub async fn unregister_mcp_tool(&self, name: &str) {
        let mut tools = self.mcp_tools.write().await;
        if tools.remove(name).is_some() {
            info!("Unregistered MCP tool: {}", name);
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
    State(state): State<Arc<WebSocketState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: Arc<WebSocketState>) {
    let (sender, mut receiver) = socket.split();
    
    let connection_id = {
        let mut connections = state.connections.write().await;
        let id = connections.len();
        connections.push(Arc::new(RwLock::new(Some(WebSocketConnection {
            id,
            sender,
        }))));
        id
    };
    
    info!("WebSocket connection established: {}", connection_id);
    
    let state_clone = state.clone();
    let send_task = tokio::spawn(async move {
        let mut interval = time::interval(state_clone.batcher.frame_duration());
        
        loop {
            interval.tick().await;
            
            let batches = state_clone.batcher.get_all_batches().await;
            if batches.is_empty() {
                continue;
            }
            
            let connections = state_clone.connections.read().await;
            if connection_id >= connections.len() {
                break;
            }
            
            let conn_lock = connections[connection_id].clone();
            drop(connections);
            
            let mut conn = conn_lock.write().await;
            if let Some(connection) = conn.as_mut() {
                for (channel_id, batch) in batches {
                    debug!("Sending batch for channel {}: {} bytes", channel_id, batch.len());
                    
                    if let Err(e) = connection.sender.send(Message::Binary(batch)).await {
                        error!("Failed to send batch: {}", e);
                        *conn = None;
                        return;
                    }
                }
            } else {
                break;
            }
        }
    });
    
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                if let Err(e) = handle_message(Bytes::from(data), &state).await {
                    error!("Error handling message: {}", e);
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection {} closing", connection_id);
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    send_task.abort();
    
    let mut connections = state.connections.write().await;
    if connection_id < connections.len() {
        *connections[connection_id].write().await = None;
    }
    
    info!("WebSocket connection {} closed", connection_id);
}

async fn handle_message(data: Bytes, state: &WebSocketState) -> anyhow::Result<()> {
    let packet = Packet::deserialize(data)?;
    
    if packet.channel_id == 0 {
        handle_control_message(packet, state).await?;
    } else {
        state.batcher.queue_packet(packet).await;
    }
    
    Ok(())
}

async fn handle_control_message(packet: Packet, state: &WebSocketState) -> anyhow::Result<()> {
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
            info!("Registered MCP tool '{}' for channel {}", name, handler_channel);
        }
        return Ok(());
    } else if packet.packet_type == 101 {
        // Unregister MCP tool
        let unregistration: serde_json::Value = serde_json::from_slice(&packet.payload)?;
        
        if let Some(name) = unregistration.get("name").and_then(|v| v.as_str()) {
            state.unregister_mcp_tool(name).await;
            info!("Unregistered MCP tool '{}'", name);
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
            
            match state.channel_manager.write().await.register_system(name.clone(), channel_id).await {
                Ok(id) => {
                    let response = create_register_response(id);
                    state.batcher.queue_packet(response).await;
                    info!("Registered system '{}' on channel {}", name, id);
                }
                Err(e) => {
                    let error = create_error_response(format!("Failed to register system: {}", e));
                    state.batcher.queue_packet(error).await;
                }
            }
        }
        ControlMessageType::RegisterPlugin => {
            let name = String::from_utf8(packet.payload.to_vec())?;
            
            match state.channel_manager.write().await.register_plugin(name.clone()).await {
                Ok(id) => {
                    let response = create_register_response(id);
                    state.batcher.queue_packet(response).await;
                    info!("Registered plugin '{}' on channel {}", name, id);
                }
                Err(e) => {
                    let error = create_error_response(format!("Failed to register plugin: {}", e));
                    state.batcher.queue_packet(error).await;
                }
            }
        }
        ControlMessageType::QueryChannel => {
            let name = String::from_utf8(packet.payload.to_vec())?;
            
            if let Some(info) = state.channel_manager.read().await.get_channel_by_name(&name).await {
                let response = create_query_response(info.id, info.name);
                state.batcher.queue_packet(response).await;
            } else {
                let error = create_error_response(format!("Channel '{}' not found", name));
                state.batcher.queue_packet(error).await;
            }
        }
        ControlMessageType::ListChannels => {
            let channels = state.channel_manager.read().await.list_channels().await;
            let response = create_list_response(channels);
            state.batcher.queue_packet(response).await;
        }
        _ => {
            debug!("Ignoring control message type: {:?}", msg_type);
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