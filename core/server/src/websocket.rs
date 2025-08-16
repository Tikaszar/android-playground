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
use tokio::sync::RwLock;
use futures_util::{StreamExt, SinkExt};
use bytes::{Bytes, BytesMut, BufMut};
use tracing::{info, error, debug};
use tokio::time;

pub struct WebSocketState {
    pub channel_manager: Arc<ChannelManager>,
    pub batcher: Arc<FrameBatcher>,
    pub connections: Arc<RwLock<Vec<Arc<RwLock<Option<WebSocketConnection>>>>>>,
}

struct WebSocketConnection {
    id: usize,
    sender: futures_util::stream::SplitSink<WebSocket, Message>,
}

impl WebSocketState {
    pub fn new() -> Self {
        Self {
            channel_manager: Arc::new(ChannelManager::new()),
            batcher: Arc::new(FrameBatcher::new(2000, 60)), // 60fps, support up to 2000 channels
            connections: Arc::new(RwLock::new(Vec::new())),
        }
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
    let msg_type = ControlMessageType::try_from(packet.packet_type)?;
    
    match msg_type {
        ControlMessageType::RegisterSystem => {
            let name = String::from_utf8(packet.payload.to_vec())?;
            let channel_id = name.split(':')
                .nth(1)
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);
            
            let name = name.split(':').next().unwrap_or(&name).to_string();
            
            match state.channel_manager.register_system(name.clone(), channel_id) {
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
            
            match state.channel_manager.register_plugin(name.clone()).await {
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
            
            if let Some(info) = state.channel_manager.get_channel_by_name(&name) {
                let response = create_query_response(info.id, info.name);
                state.batcher.queue_packet(response).await;
            } else {
                let error = create_error_response(format!("Channel '{}' not found", name));
                state.batcher.queue_packet(error).await;
            }
        }
        ControlMessageType::ListChannels => {
            let channels = state.channel_manager.list_channels();
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