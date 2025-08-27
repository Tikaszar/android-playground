use playground_core_types::{Handle, handle};
use bytes::Bytes;
use async_trait::async_trait;
use playground_core_ecs::{MessageBus, MessageHandler, MessageHandlerData, BroadcasterData, ChannelId, EcsResult};
use crate::websocket::WebSocketState;
use crate::packet::Packet;
use futures_util::SinkExt;
use axum::extract::ws::Message;

/// Bridge between internal ECS messaging and WebSocket clients
pub struct MessageBridge {
    ecs_bus: Handle<MessageBus>,
    websocket_state: Handle<WebSocketState>,
}

impl MessageBridge {
    /// Create a new message bridge
    pub fn new(ecs_bus: Handle<MessageBus>, websocket_state: Handle<WebSocketState>) -> Self {
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
        let forwarder = WebSocketForwarder {
            channel: internal_channel,
            websocket_state: ws_state,
        };
        
        let handler = MessageHandler::new(forwarder).await
            .expect("Failed to create handler");
        
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
    websocket_state: Handle<WebSocketState>,
}

#[async_trait]
impl MessageHandlerData for WebSocketForwarder {
    fn handler_id(&self) -> String {
        format!("websocket_forwarder_{}", self.channel)
    }
    
    async fn handle(&self, _channel: ChannelId, message: Bytes) -> EcsResult<()> {
        // Create a packet from the message
        let packet = Packet::new(
            self.channel,
            104, // RenderBatch type for UI channel (we should make this configurable)
            playground_core_types::Priority::High,
            message,
        );
        
        // Queue the packet in the batcher for frame-based broadcast
        self.websocket_state.batcher.queue_packet(packet).await;
        
        // Log that we queued the packet
        self.websocket_state.dashboard.log(
            crate::dashboard::LogLevel::Debug,
            format!("MessageBridge: Queued packet on channel {} for broadcast", self.channel),
            None
        ).await;
        
        Ok(())
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        // Can't really serialize a WebSocket connection
        Ok(Bytes::from(format!("websocket_forwarder_{}", self.channel)))
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(playground_core_ecs::EcsError::MessageError(
            "WebSocketForwarder cannot be deserialized".to_string()
        ))
    }
}

/// WebSocket to ECS broadcaster
/// This allows the server to act as a Broadcaster for the ECS MessageBus
pub struct WebSocketBroadcaster {
    websocket_state: Handle<WebSocketState>,
}

impl WebSocketBroadcaster {
    pub fn new(websocket_state: Handle<WebSocketState>) -> Self {
        Self { websocket_state }
    }
}

#[async_trait]
impl BroadcasterData for WebSocketBroadcaster {
    fn broadcaster_id(&self) -> String {
        "websocket_broadcaster".to_string()
    }
    
    async fn broadcast(&self, channel: ChannelId, message: Bytes) -> EcsResult<()> {
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
    
    async fn send_to(&self, _target: playground_core_ecs::EntityId, _message: Bytes) -> EcsResult<()> {
        // Not implemented for WebSocket broadcast
        // Could be used for sending to specific clients in the future
        Ok(())
    }
}