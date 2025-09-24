//! Generic types for client operations

use serde::{Deserialize, Serialize};

/// Type aliases for consistency
pub type Float = f32;
pub type Int = i32;
pub type UInt = u32;

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
    pub max_resolution: Option<(UInt, UInt)>,
    /// Supports multiple windows/surfaces
    pub multi_window: bool,
    /// Platform name (generic)
    pub platform: String,
}

impl Default for ClientCapabilities {
    fn default() -> Self {
        Self {
            can_render: cfg!(feature = "rendering"),
            can_play_audio: cfg!(feature = "audio"),
            can_input: cfg!(feature = "input"),
            render_backends: Vec::new(),
            max_resolution: None,
            multi_window: false,
            platform: "unknown".to_string(),
        }
    }
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
    pub reconnect_delay_ms: UInt,
    /// Maximum reconnect attempts (0 = infinite)
    pub max_reconnect_attempts: UInt,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            id: ClientId(0),
            name: "Generic Client".to_string(),
            version: "1.0.0".to_string(),
            auto_reconnect: true,
            reconnect_delay_ms: 1000,
            max_reconnect_attempts: 5,
            capabilities: ClientCapabilities::default(),
        }
    }
}

/// Render target information (generic, not graphics API specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTarget {
    /// Unique ID for this render target
    pub id: UInt,
    /// Width in pixels
    pub width: UInt,
    /// Height in pixels
    pub height: UInt,
    /// Pixel density/DPI scale factor
    pub scale_factor: Float,
    /// Is this the primary/main target
    pub is_primary: bool,
    /// Generic properties
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Client statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStats {
    /// Current frames per second
    pub fps: Float,
    /// Average frame time in milliseconds
    pub frame_time_ms: Float,
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

impl Default for ClientStats {
    fn default() -> Self {
        Self {
            fps: 0.0,
            frame_time_ms: 0.0,
            total_frames: 0,
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            memory_usage: 0,
        }
    }
}