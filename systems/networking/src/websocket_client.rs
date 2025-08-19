use crate::{NetworkError, NetworkResult};
use bytes::{Bytes, BytesMut, BufMut};
use futures_util::{SinkExt, StreamExt};
use playground_server::packet::{Packet, Priority};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// Native WebSocket client for connecting to core/server
pub struct WebSocketClient {
    url: String,
    tx: mpsc::UnboundedSender<Packet>,
    rx: Arc<RwLock<mpsc::UnboundedReceiver<Packet>>>,
    incoming_tx: mpsc::UnboundedSender<Packet>,
    incoming_rx: Arc<RwLock<mpsc::UnboundedReceiver<Packet>>>,
    connected: Arc<RwLock<bool>>,
}

impl WebSocketClient {
    pub fn new(url: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        
        Self {
            url,
            tx,
            rx: Arc::new(RwLock::new(rx)),
            incoming_tx,
            incoming_rx: Arc::new(RwLock::new(incoming_rx)),
            connected: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Connect to the WebSocket server
    pub async fn connect(&self) -> NetworkResult<()> {
        info!("Connecting to WebSocket at {}", self.url);
        
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
        
        info!("WebSocket connected successfully");
        
        let (mut write, mut read) = ws_stream.split();
        
        // Mark as connected
        *self.connected.write().await = true;
        
        // Spawn send task
        let rx = self.rx.clone();
        let connected = self.connected.clone();
        tokio::spawn(async move {
            let mut rx = rx.write().await;
            
            while let Some(packet) = rx.recv().await {
                // Serialize packet to binary
                let binary = serialize_packet(&packet);
                
                if let Err(e) = write.send(Message::Binary(binary)).await {
                    error!("Failed to send packet: {}", e);
                    *connected.write().await = false;
                    break;
                }
            }
        });
        
        // Spawn receive task
        let incoming_tx = self.incoming_tx.clone();
        let connected = self.connected.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Binary(data)) => {
                        match deserialize_packet(&data) {
                            Ok(packet) => {
                                if incoming_tx.send(packet).is_err() {
                                    error!("Failed to queue incoming packet");
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to deserialize packet: {}", e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket closed");
                        *connected.write().await = false;
                        break;
                    }
                    Ok(_) => {
                        // Ignore text messages and other types
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        *connected.write().await = false;
                        break;
                    }
                }
            }
        });
        
        // Send initial registration for control channel
        self.register_control_channel().await?;
        
        Ok(())
    }
    
    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
    
    /// Send a packet
    pub async fn send_packet(&self, packet: Packet) -> NetworkResult<()> {
        if !self.is_connected().await {
            return Err(NetworkError::NotConnected);
        }
        
        self.tx.send(packet)
            .map_err(|_| NetworkError::ChannelError("Failed to queue packet".to_string()))?;
        
        Ok(())
    }
    
    /// Receive incoming packets
    pub async fn receive_packets(&self) -> Vec<Packet> {
        let mut packets = Vec::new();
        let mut rx = self.incoming_rx.write().await;
        
        // Drain all available packets
        while let Ok(packet) = rx.try_recv() {
            packets.push(packet);
        }
        
        packets
    }
    
    /// Register for the control channel
    async fn register_control_channel(&self) -> NetworkResult<()> {
        debug!("Registering for control channel");
        
        // Create control registration packet
        let registration = serde_json::json!({
            "type": 0, // CONTROL_REGISTER
            "channel_id": 0,
            "name": "networking-system"
        });
        
        let payload = serde_json::to_vec(&registration)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;
        
        let packet = Packet {
            channel_id: 0,
            packet_type: 0, // Control packet
            priority: Priority::High,
            payload: Bytes::from(payload),
        };
        
        self.send_packet(packet).await?;
        
        Ok(())
    }
    
    /// Register for a specific channel
    pub async fn register_channel(&self, channel_id: u16, name: &str) -> NetworkResult<()> {
        debug!("Registering channel {}: {}", channel_id, name);
        
        let registration = serde_json::json!({
            "type": 0, // CONTROL_REGISTER
            "channel_id": channel_id,
            "name": name
        });
        
        let payload = serde_json::to_vec(&registration)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;
        
        let packet = Packet {
            channel_id: 0, // Control channel
            packet_type: 0,
            priority: Priority::High,
            payload: Bytes::from(payload),
        };
        
        self.send_packet(packet).await?;
        
        Ok(())
    }
}

/// Serialize a packet to binary format
fn serialize_packet(packet: &Packet) -> Vec<u8> {
    let payload_len = packet.payload.len() as u32;
    let mut buffer = BytesMut::with_capacity(9 + payload_len as usize);
    
    // Write header
    buffer.put_u16_le(packet.channel_id);
    buffer.put_u16_le(packet.packet_type);
    buffer.put_u8(packet.priority as u8);
    buffer.put_u32_le(payload_len);
    
    // Write payload
    buffer.extend_from_slice(&packet.payload);
    
    buffer.to_vec()
}

/// Deserialize a packet from binary format
fn deserialize_packet(data: &[u8]) -> NetworkResult<Packet> {
    if data.len() < 9 {
        return Err(NetworkError::InvalidMessage("Packet too short".to_string()));
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
    let payload_len = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
    
    if data.len() < 9 + payload_len {
        return Err(NetworkError::InvalidMessage("Payload truncated".to_string()));
    }
    
    let payload = Bytes::from(data[9..9 + payload_len].to_vec());
    
    Ok(Packet {
        channel_id,
        packet_type,
        priority,
        payload,
    })
}