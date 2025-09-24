//! Generic types for server operations
//! 
//! These types are generic and can be used by any server implementation
//! (WebSocket, TCP, UDP, IPC, named pipes, etc.)

use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::collections::HashMap;

/// Generic message that can be sent over any transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: MessageId,
    /// Channel this message belongs to (logical grouping)
    pub channel: ChannelId,
    /// Priority for ordering/QoS
    pub priority: MessagePriority,
    /// The actual message payload
    pub payload: Vec<u8>,
    /// Optional correlation ID for request/response patterns
    pub correlation_id: Option<MessageId>,
}

/// Unique identifier for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub u64);

/// Message priority levels (generic, not protocol-specific)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Generic connection identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub usize);

impl ConnectionId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Information about a connection (generic, not protocol-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: ConnectionId,
    /// Timestamp in seconds since UNIX epoch
    pub established_at: u64,
    /// Timestamp in seconds since UNIX epoch
    pub last_activity: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub status: ConnectionStatus,
    /// Generic metadata (could be IP, pipe name, etc.)
    pub metadata: HashMap<String, String>,
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
    Error,
}

/// Channel identifier for logical message grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub u16);

impl ChannelId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU16, Ordering};
        static COUNTER: AtomicU16 = AtomicU16::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Information about a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub id: ChannelId,
    pub name: String,
    pub description: Option<String>,
    /// Timestamp in seconds since UNIX epoch
    pub created_at: u64,
    pub message_count: u64,
    pub subscriber_count: usize,
}

/// Generic server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Maximum number of connections (0 = unlimited)
    pub max_connections: usize,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Keep-alive interval (None = disabled)
    pub keep_alive_interval: Option<Duration>,
    /// Message queue size per connection
    pub message_queue_size: usize,
    /// Enable message batching
    pub enable_batching: bool,
    /// Batch interval (if batching enabled)
    pub batch_interval: Duration,
    /// Generic configuration options
    pub options: HashMap<String, serde_json::Value>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            max_message_size: 1024 * 1024, // 1MB
            connection_timeout: Duration::from_secs(30),
            keep_alive_interval: Some(Duration::from_secs(30)),
            message_queue_size: 1000,
            enable_batching: true,
            batch_interval: Duration::from_millis(16), // ~60fps
            options: HashMap::new(),
        }
    }
}

/// Server statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    /// Timestamp in seconds since UNIX epoch
    pub start_time: u64,
    pub total_connections: u64,
    pub active_connections: usize,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub errors: u64,
}

impl Default for ServerStats {
    fn default() -> Self {
        Self {
            start_time: 0,
            total_connections: 0,
            active_connections: 0,
            total_messages_sent: 0,
            total_messages_received: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            errors: 0,
        }
    }
}