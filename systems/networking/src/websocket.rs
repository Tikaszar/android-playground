use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use bytes::Bytes;
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use playground_core_server::{
    WebSocketContract, ConnectionHandle, Packet, Priority, ClientInfo, ClientStatus
};
use playground_core_ecs::{MessageHandlerData, MessageBusContract, ChannelId, EcsResult, EcsError};
use playground_core_types::{Shared, shared};
use tokio::sync::mpsc;
use std::time::Instant;

/// WebSocket handler that IS a MessageHandler in the unified system
pub struct WebSocketHandler {
    connections: Shared<HashMap<usize, ConnectionState>>,
    next_connection_id: Shared<usize>,
    message_bus: Shared<Option<Arc<dyn MessageBusContract>>>,
}

struct ConnectionState {
    handle: ConnectionHandle,
    sender: mpsc::Sender<Message>,
}

impl WebSocketHandler {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            connections: shared(HashMap::new()),
            next_connection_id: shared(1),
            message_bus: shared(None),
        })
    }
    
    pub async fn connect_to_message_bus(&self, bus: Arc<dyn MessageBusContract>) -> Result<(), Box<dyn std::error::Error>> {
        let mut message_bus = self.message_bus.write().await;
        *message_bus = Some(bus.clone());
        
        // Subscribe to channels we want to forward to clients
        // Channel 10 is UI render channel
        bus.subscribe(10, self.handler_id()).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        // Subscribe to other system channels as needed
        // This replaces the old bridge functionality
        
        Ok(())
    }
}

// Clone implementation for Arc wrapping
impl Clone for WebSocketHandler {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            next_connection_id: self.next_connection_id.clone(),
            message_bus: self.message_bus.clone(),
        }
    }
}

#[async_trait]
impl WebSocketContract for WebSocketHandler {
    async fn add_connection(&self, mut conn: ConnectionHandle) -> Result<(), Box<dyn std::error::Error>> {
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
    
    async fn remove_connection(&self, id: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut connections = self.connections.write().await;
        connections.remove(&id);
        Ok(())
    }
    
    async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
    
    async fn broadcast(&self, packet: Packet) -> Result<(), Box<dyn std::error::Error>> {
        let connections = self.connections.read().await;
        let binary_data = serialize_packet(&packet)?;
        let message = Message::Binary(Bytes::from(binary_data));
        
        for (_, conn) in connections.iter() {
            let _ = conn.sender.send(message.clone()).await;
        }
        
        Ok(())
    }
    
    async fn send_to(&self, conn_id: usize, packet: Packet) -> Result<(), Box<dyn std::error::Error>> {
        let connections = self.connections.read().await;
        
        if let Some(conn) = connections.get(&conn_id) {
            let binary_data = serialize_packet(&packet)?;
            let message = Message::Binary(Bytes::from(binary_data));
            conn.sender.send(message).await?;
        }
        
        Ok(())
    }
}

// This makes WebSocket a MessageHandler in the unified ECS messaging system!
#[async_trait]
impl MessageHandlerData for WebSocketHandler {
    fn handler_id(&self) -> String {
        "WebSocketHandler".to_string()
    }
    
    async fn handle(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
        // When we receive a message from the ECS MessageBus,
        // forward it to all WebSocket clients
        
        let packet = Packet {
            channel_id: channel,
            packet_type: 0, // Default type for ECS messages
            priority: Priority::Medium,
            payload: message.to_vec(),
        };
        
        self.broadcast(packet).await
            .map_err(|e| EcsError::Generic(e.to_string()))?;
        
        Ok(())
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        // WebSocket handler doesn't need serialization
        Ok(Bytes::new())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        // Create a new WebSocket handler
        WebSocketHandler::new().await
            .map_err(|e| EcsError::Generic(e.to_string()))
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
                if let Ok(packet) = deserialize_packet(&data) {
                    // Route to appropriate handler based on channel
                    // This is where incoming messages from clients are processed
                    
                    // If we have a message bus, publish to it
                    let bus_opt = handler.message_bus.read().await;
                    if let Some(ref bus) = *bus_opt {
                        let _ = bus.publish(packet.channel_id, Bytes::from(packet.payload)).await;
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

fn serialize_packet(packet: &Packet) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    data.extend_from_slice(&packet.channel_id.to_le_bytes());
    data.extend_from_slice(&packet.packet_type.to_le_bytes());
    data.push(packet.priority.clone() as u8);
    data.extend_from_slice(&[0u8; 3]); // Reserved
    data.extend_from_slice(&(packet.payload.len() as u32).to_le_bytes());
    data.extend_from_slice(&packet.payload);
    Ok(data)
}

fn deserialize_packet(data: &[u8]) -> Result<Packet, String> {
    if data.len() < 12 {
        return Err("Packet too small".into());
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