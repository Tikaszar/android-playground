//! WebSocket handler implementation
//!
//! This module handles WebSocket connections without using old trait systems.
//! All functionality is exposed through VTable handlers in vtable_handlers.rs

use std::sync::Arc;
use std::collections::HashMap;
use bytes::Bytes;
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use playground_core_types::{Shared, shared, CoreResult, CoreError};
use playground_core_server::{ConnectionId, ConnectionInfo, ConnectionStatus};
use tokio::sync::mpsc;
use std::time::Instant;
use crate::types::{Packet, Priority, ClientInfo, ClientStatus, ConnectionHandle};

/// WebSocket handler for managing connections
pub struct WebSocketHandler {
    connections: Shared<HashMap<usize, ConnectionState>>,
    next_connection_id: Shared<usize>,
}

struct ConnectionState {
    handle: ConnectionHandle,
    sender: mpsc::Sender<Message>,
}

impl WebSocketHandler {
    pub async fn new() -> CoreResult<Self> {
        Ok(Self {
            connections: shared(HashMap::new()),
            next_connection_id: shared(1),
        })
    }
    
    pub async fn add_connection(&self, mut conn: ConnectionHandle) -> CoreResult<()> {
        let (tx, mut rx) = mpsc::channel(100);
        
        let conn_id = conn.id;
        let state = ConnectionState {
            handle: ConnectionHandle {
                id: conn_id,
                sender: conn.sender.clone(),
                info: conn.info.clone(),
            },
            sender: tx,
        };
        
        {
            let mut connections = self.connections.write().await;
            connections.insert(conn_id, state);
        }
        
        // Spawn task to handle outgoing messages for this connection
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                // Convert to bytes and send via the connection's sender
                if let Message::Binary(data) = msg {
                    let _ = conn.sender.send(Bytes::from(data)).await;
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn remove_connection(&self, id: usize) -> CoreResult<()> {
        let mut connections = self.connections.write().await;
        connections.remove(&id);
        Ok(())
    }
    
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
    
    pub async fn store_connection(&self, info: ConnectionInfo) {
        // Convert core ConnectionInfo to our ClientInfo
        // This is used by vtable_handlers.rs
    }
    
    pub async fn remove_connection_by_core_id(&self, id: ConnectionId) {
        let mut connections = self.connections.write().await;
        connections.remove(&id.0);
    }
    
    pub async fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.iter().map(|(id, state)| {
            let mut metadata = HashMap::new();
            metadata.insert("ip".to_string(), state.handle.info.ip_address.clone());
            if let Some(ref ua) = state.handle.info.user_agent {
                metadata.insert("user_agent".to_string(), ua.clone());
            }
            
            ConnectionInfo {
                id: ConnectionId(*id),
                established_at: 0, // Would need to track this properly
                last_activity: 0,
                bytes_sent: state.handle.info.bytes_sent,
                bytes_received: state.handle.info.bytes_received,
                messages_sent: state.handle.info.messages_sent,
                messages_received: state.handle.info.messages_received,
                status: match state.handle.info.status {
                    ClientStatus::Connected => ConnectionStatus::Connected,
                    ClientStatus::Connecting => ConnectionStatus::Connecting,
                    ClientStatus::Disconnecting => ConnectionStatus::Disconnecting,
                    ClientStatus::Disconnected => ConnectionStatus::Disconnected,
                },
                metadata,
            }
        }).collect()
    }
    
    pub async fn get_connection(&self, id: ConnectionId) -> Option<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.get(&id.0).map(|state| {
            let mut metadata = HashMap::new();
            metadata.insert("ip".to_string(), state.handle.info.ip_address.clone());
            if let Some(ref ua) = state.handle.info.user_agent {
                metadata.insert("user_agent".to_string(), ua.clone());
            }
            
            ConnectionInfo {
                id,
                established_at: 0,
                last_activity: 0,
                bytes_sent: state.handle.info.bytes_sent,
                bytes_received: state.handle.info.bytes_received,
                messages_sent: state.handle.info.messages_sent,
                messages_received: state.handle.info.messages_received,
                status: match state.handle.info.status {
                    ClientStatus::Connected => ConnectionStatus::Connected,
                    ClientStatus::Connecting => ConnectionStatus::Connecting,
                    ClientStatus::Disconnecting => ConnectionStatus::Disconnecting,
                    ClientStatus::Disconnected => ConnectionStatus::Disconnected,
                },
                metadata,
            }
        })
    }
    
    pub async fn broadcast(&self, packet: Packet) -> CoreResult<()> {
        let connections = self.connections.read().await;
        let binary_data = serialize_packet(&packet)?;
        let message = Message::Binary(Bytes::from(binary_data));
        
        for (_, conn) in connections.iter() {
            let _ = conn.sender.send(message.clone()).await;
        }
        
        Ok(())
    }
    
    pub async fn send_to(&self, conn_id: usize, packet: Packet) -> CoreResult<()> {
        let connections = self.connections.read().await;
        
        if let Some(conn) = connections.get(&conn_id) {
            let binary_data = serialize_packet(&packet)?;
            let message = Message::Binary(Bytes::from(binary_data));
            conn.sender.send(message).await
                .map_err(|e| CoreError::Network(e.to_string()))?;
        }
        
        Ok(())
    }
}

// Clone implementation for Arc wrapping
impl Clone for WebSocketHandler {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            next_connection_id: self.next_connection_id.clone(),
        }
    }
}

/// Axum handler for WebSocket upgrade
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(handler): State<Arc<WebSocketHandler>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, handler))
}

async fn handle_socket(socket: WebSocket, handler: Arc<WebSocketHandler>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(100);
    
    // Generate connection ID
    let conn_id = {
        let mut next_id = handler.next_connection_id.write().await;
        let id = *next_id;
        *next_id += 1;
        id
    };
    
    // Create connection handle
    let (bytes_tx, mut bytes_rx) = mpsc::channel::<Bytes>(100);
    let conn_handle = ConnectionHandle {
        id: conn_id,
        sender: bytes_tx,
        info: ClientInfo {
            id: conn_id,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            ip_address: "unknown".to_string(),
            user_agent: None,
            status: ClientStatus::Connected,
        },
    };
    
    // Add connection
    let _ = handler.add_connection(conn_handle).await;
    
    // Spawn task to handle outgoing messages
    let handler_clone = handler.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
        let _ = handler_clone.remove_connection(conn_id).await;
    });
    
    // Spawn task to convert bytes to messages
    tokio::spawn(async move {
        while let Some(bytes) = bytes_rx.recv().await {
            let _ = tx.send(Message::Binary(bytes)).await;
        }
    });
    
    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Binary(data) => {
                // Parse packet and handle
                match deserialize_packet(&data) {
                    Ok(packet) => {
                        // Route to appropriate handler based on channel
                        // This is where incoming messages from clients are processed
                        // The actual routing is handled by channel subscriptions
                    }
                    Err(_) => {
                        // Invalid packet, ignore
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    
    // Clean up connection
    let _ = handler.remove_connection(conn_id).await;
}

fn serialize_packet(packet: &Packet) -> CoreResult<Vec<u8>> {
    let mut data = Vec::new();
    data.extend_from_slice(&packet.channel_id.to_le_bytes());
    data.extend_from_slice(&packet.packet_type.to_le_bytes());
    data.push(packet.priority.clone() as u8);
    data.extend_from_slice(&[0u8; 3]); // Reserved
    data.extend_from_slice(&(packet.payload.len() as u32).to_le_bytes());
    data.extend_from_slice(&packet.payload);
    Ok(data)
}

fn deserialize_packet(data: &[u8]) -> CoreResult<Packet> {
    if data.len() < 12 {
        return Err(CoreError::InvalidInput("Packet too small".into()));
    }
    
    let channel_id = u16::from_le_bytes([data[0], data[1]]);
    let packet_type = u16::from_le_bytes([data[2], data[3]]);
    let priority = match data[4] {
        0 => Priority::Low,
        1 => Priority::Medium,
        2 => Priority::High,
        3 => Priority::Critical,
        4 => Priority::Blocker,
        _ => Priority::Medium,
    };
    
    let payload_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    let payload = data[12..12 + payload_len].to_vec();
    
    Ok(Packet {
        channel_id,
        packet_type,
        priority,
        payload,
    })
}