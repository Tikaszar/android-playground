use std::sync::Arc;
use tokio::sync::RwLock;
use bytes::Bytes;
use async_trait::async_trait;
use playground_core_ecs::{MessageBus, MessageHandler, Broadcaster, ChannelId};
use crate::websocket::WebSocketState;
use crate::packet::Packet;
use futures_util::SinkExt;
use axum::extract::ws::Message;

/// Bridge between internal ECS messaging and WebSocket clients
pub struct MessageBridge {
    ecs_bus: Arc<MessageBus>,
    websocket_state: Arc<WebSocketState>,
}

impl MessageBridge {
    /// Create a new message bridge
    pub fn new(ecs_bus: Arc<MessageBus>, websocket_state: Arc<WebSocketState>) -> Self {
        Self {
            ecs_bus,
            websocket_state,
        }
    }
    
    /// Bridge an internal channel to WebSocket clients
    /// Messages published to internal_channel will be forwarded to all WebSocket clients
    pub async fn bridge_channel(&self, internal_channel: ChannelId) {
        let ws_state = self.websocket_state.clone();
        
        // Create a handler that forwards to WebSocket
        let handler = Arc::new(WebSocketForwarder {
            channel: internal_channel,
            websocket_state: ws_state,
        });
        
        // Subscribe to the internal channel
        self.ecs_bus.subscribe(internal_channel, handler).await
            .expect("Failed to subscribe to channel");
    }
    
    /// Setup all standard channel bridges
    pub async fn setup_standard_bridges(&self) {
        // Bridge UI render channel (10) to WebSocket clients
        self.bridge_channel(10).await;
        
        // Bridge other system channels as needed
        // self.bridge_channel(100).await; // Networking
        // self.bridge_channel(200).await; // Physics
        // etc.
    }
}

/// Handler that forwards messages to WebSocket clients
struct WebSocketForwarder {
    channel: ChannelId,
    websocket_state: Arc<WebSocketState>,
}

#[async_trait]
impl MessageHandler for WebSocketForwarder {
    async fn handle(&self, _channel: ChannelId, message: Bytes) -> playground_core_ecs::EcsResult<()> {
        // Create a packet from the message
        let packet = Packet::new(
            self.channel,
            104, // RenderBatch type for UI channel (we should make this configurable)
            playground_core_types::Priority::High,
            message,
        );
        
        // Broadcast to all WebSocket clients
        let connections = self.websocket_state.connections.read().await;
        
        // Log the broadcast
        self.websocket_state.dashboard.log(
            crate::dashboard::LogLevel::Debug,
            format!("MessageBridge: Broadcasting to {} WebSocket clients on channel {}", 
                connections.len(), self.channel),
            None
        ).await;
        
        for (conn_id, conn_lock) in connections.iter().enumerate() {
            let mut conn = conn_lock.write().await;
            if let Some(connection) = conn.as_mut() {
                let packet_bytes = packet.serialize();
                if let Err(e) = connection.send(Message::Binary(packet_bytes)).await {
                    self.websocket_state.dashboard.log(
                        crate::dashboard::LogLevel::Error,
                        format!("Failed to send to client {}: {}", conn_id, e),
                        Some(conn_id)
                    ).await;
                }
            }
        }
        
        Ok(())
    }
}

/// WebSocket to ECS broadcaster
/// This allows the server to act as a Broadcaster for the ECS MessageBus
pub struct WebSocketBroadcaster {
    websocket_state: Arc<WebSocketState>,
}

impl WebSocketBroadcaster {
    pub fn new(websocket_state: Arc<WebSocketState>) -> Self {
        Self { websocket_state }
    }
}

#[async_trait]
impl Broadcaster for WebSocketBroadcaster {
    async fn broadcast(&self, channel: ChannelId, message: Bytes) -> playground_core_ecs::EcsResult<()> {
        // Create a packet
        let packet = Packet::new(
            channel,
            104, // Should be configurable based on channel
            playground_core_types::Priority::High,
            message,
        );
        
        // Send to all WebSocket clients
        let connections = self.websocket_state.connections.read().await;
        for (conn_id, conn_lock) in connections.iter().enumerate() {
            let mut conn = conn_lock.write().await;
            if let Some(connection) = conn.as_mut() {
                let packet_bytes = packet.serialize();
                if let Err(_e) = connection.send(Message::Binary(packet_bytes)).await {
                    // Connection failed, mark as None
                    *conn = None;
                }
            }
        }
        
        Ok(())
    }
    
    async fn send_to(&self, _target: playground_core_ecs::EntityId, _message: Bytes) -> playground_core_ecs::EcsResult<()> {
        // Not implemented for WebSocket broadcast
        // Could be used for sending to specific clients in the future
        Ok(())
    }
}