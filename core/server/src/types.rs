use std::time::Instant;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// Binary packet structure for network communication
#[derive(Clone, Debug)]
pub struct Packet {
    pub channel_id: u16,
    pub packet_type: u16,
    pub priority: Priority,
    pub payload: Vec<u8>,
}

/// Priority levels for packet ordering
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
    Blocker = 4,
}

/// Logging severity levels
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    pub fn as_emoji(&self) -> &str {
        match self {
            LogLevel::Debug => "🔍",
            LogLevel::Info => "ℹ️",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Critical => "🔴",
        }
    }
    
    pub fn as_color_code(&self) -> &str {
        match self {
            LogLevel::Debug => "\x1b[90m",    // Gray
            LogLevel::Info => "\x1b[36m",     // Cyan
            LogLevel::Warning => "\x1b[33m",  // Yellow
            LogLevel::Error => "\x1b[31m",    // Red
            LogLevel::Critical => "\x1b[91m",  // Bright Red
        }
    }
}

/// Channel categorization
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChannelType {
    System,   // Core systems (1-999)
    Plugin,   // Plugin channels (1000+)
    Session,  // Dynamic sessions (2000+)
}

/// Client connection information
#[derive(Clone, Debug)]
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

/// Client connection status
#[derive(Clone, Debug, PartialEq)]
pub enum ClientStatus {
    Connecting,
    Connected,
    Idle,
    Active,
    Disconnecting,
    Disconnected,
}

impl ClientStatus {
    pub fn as_emoji(&self) -> &str {
        match self {
            ClientStatus::Connecting => "🔄",
            ClientStatus::Connected => "✅",
            ClientStatus::Idle => "💤",
            ClientStatus::Active => "🟢",
            ClientStatus::Disconnecting => "🔻",
            ClientStatus::Disconnected => "❌",
        }
    }
}

/// Channel manifest for discovery protocol
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelManifest {
    #[serde(rename = "type")]
    pub manifest_type: String,
    pub channels: HashMap<String, u16>,
}

impl ChannelManifest {
    pub fn new() -> Self {
        Self {
            manifest_type: "channel_manifest".to_string(),
            channels: HashMap::new(),
        }
    }
}

/// MCP tool definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub handler_channel: u16,
}

/// MCP request structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub method: String,
    pub params: Option<Value>,
}

/// MCP response structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

/// MCP error structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// Dashboard channel info
#[derive(Clone, Debug)]
pub struct DashboardChannelInfo {
    pub name: String,
    pub channel_id: u16,
    pub channel_type: ChannelType,
    pub registered_at: Instant,
    pub message_count: u64,
}

/// Network statistics
#[derive(Clone, Debug, Default)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub connections_active: usize,
    pub average_latency_ms: u32,
}

/// Server configuration
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub dashboard_enabled: bool,
    pub mcp_enabled: bool,
    pub frame_rate: u32,
    pub max_connections: usize,
    pub log_to_file: bool,
}

/// Handle for a WebSocket connection
pub struct ConnectionHandle {
    pub id: usize,
    pub sender: tokio::sync::mpsc::Sender<bytes::Bytes>,
    pub info: ClientInfo,
}