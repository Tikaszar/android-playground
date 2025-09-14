//! Generic types for client operations

use serde::{Deserialize, Serialize};

/// Unique identifier for a client instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

/// Client lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientState {
    /// Client is initializing
    Initializing,
    /// Client is ready but not connected
    Ready,
    /// Client is connecting to server
    Connecting,
    /// Client is connected and active
    Connected,
    /// Client is disconnecting
    Disconnecting,
    /// Client is disconnected
    Disconnected,
    /// Client encountered an error
    Error,
}

/// Client capabilities (what this client can do)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Can render graphics
    pub can_render: bool,
    /// Can play audio
    pub can_play_audio: bool,
    /// Can accept input
    pub can_input: bool,
    /// Supported rendering backends
    pub render_backends: Vec<String>,
    /// Maximum render resolution (width, height)
    pub max_resolution: Option<(u32, u32)>,
    /// Supports multiple windows/surfaces
    pub multi_window: bool,
    /// Platform name (generic)
    pub platform: String,
}

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Unique client ID
    pub id: ClientId,
    /// Human-readable client name
    pub name: String,
    /// Client version
    pub version: String,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// Reconnect delay in milliseconds
    pub reconnect_delay_ms: u32,
    /// Maximum reconnect attempts (0 = infinite)
    pub max_reconnect_attempts: u32,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
}

/// Render target information (generic, not graphics API specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTarget {
    /// Unique ID for this render target
    pub id: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Pixel density/DPI scale factor
    pub scale_factor: f32,
    /// Is this the primary/main target
    pub is_primary: bool,
    /// Generic properties
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Client statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStats {
    /// Current frames per second
    pub fps: f32,
    /// Average frame time in milliseconds
    pub frame_time_ms: f32,
    /// Total frames rendered
    pub total_frames: u64,
    /// Messages sent to server
    pub messages_sent: u64,
    /// Messages received from server
    pub messages_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Current memory usage in bytes
    pub memory_usage: u64,
}