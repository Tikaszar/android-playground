//! WebSocket/HTTP specific types for the networking implementation
//!
//! These types are specific to the WebSocket/HTTP implementation of the generic
//! server contracts. They are NOT part of the core contracts.

use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::collections::HashMap;
use bytes::Bytes;
use tokio::sync::mpsc;

/// WebSocket-specific packet structure for binary protocol
#[derive(Debug, Clone)]
pub struct Packet {
    pub channel_id: u16,
    pub packet_type: u16,
    pub priority: Priority,
    pub payload: Vec<u8>,
}

/// WebSocket packet priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
    Blocker,
}

/// WebSocket client information
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: usize,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub status: ClientStatus,
}

/// WebSocket client status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientStatus {
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
}

/// WebSocket connection handle
pub struct ConnectionHandle {
    pub id: usize,
    pub sender: mpsc::Sender<Bytes>,
    pub info: ClientInfo,
}

/// Channel manifest for dynamic channel discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelManifest {
    pub channels: HashMap<String, u16>,
}

impl ChannelManifest {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }
}

/// MCP tool definition for AI/LLM integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub handler_channel: u16,
}

/// Log level for console logging (temporary until we use command processor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// WebSocket-specific server configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub port: u16,
    pub frame_rate: u32,
    pub max_connections: usize,
    pub max_message_size: usize,
    pub mcp_enabled: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            frame_rate: 60,
            max_connections: 100,
            max_message_size: 1024 * 1024, // 1MB
            mcp_enabled: true,
        }
    }
}