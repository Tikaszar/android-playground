//! Concrete Server data structure - NO LOGIC, just data fields!
//! 
//! This is like an abstract base class - defines structure only.
//! All actual implementation logic is in systems/networking.

use std::collections::HashMap;
use playground_core_types::{Handle, handle, Shared, shared};
use playground_core_ecs::VTable;
use crate::types::*;

/// The concrete Server struct - data fields only, no logic!
/// 
/// Like an abstract base class in OOP - structure but no behavior.
/// All actual server operations are implemented in systems/networking.
pub struct Server {
    /// The VTable for system dispatch
    pub vtable: VTable,
    
    /// Server configuration
    pub config: Shared<ServerConfig>,
    
    /// Server statistics
    pub stats: Shared<ServerStats>,
    
    /// Active connections
    pub connections: Shared<HashMap<ConnectionId, ConnectionInfo>>,
    
    /// Channel registry
    #[cfg(feature = "channels")]
    pub channels: Shared<HashMap<ChannelId, ChannelInfo>>,
    
    /// Channel subscriptions: channel_id -> list of connection_ids
    #[cfg(feature = "channels")]
    pub subscriptions: Shared<HashMap<ChannelId, Vec<ConnectionId>>>,
    
    /// Message queue for batching
    #[cfg(feature = "batching")]
    pub message_queue: Shared<Vec<(ConnectionId, Message)>>,
    
    /// Server running state
    pub is_running: Shared<bool>,
    
    /// Server capabilities
    pub capabilities: ServerCapabilities,
}

/// Server capabilities (feature-based)
#[derive(Debug, Clone)]
pub struct ServerCapabilities {
    pub supports_websocket: bool,
    pub supports_tcp: bool,
    pub supports_udp: bool,
    pub supports_ipc: bool,
    pub supports_channels: bool,
    pub supports_batching: bool,
    pub supports_compression: bool,
    pub supports_encryption: bool,
    pub max_message_size: usize,
    pub max_connections: usize,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            supports_websocket: cfg!(feature = "websocket"),
            supports_tcp: cfg!(feature = "tcp"),
            supports_udp: cfg!(feature = "udp"),
            supports_ipc: cfg!(feature = "ipc"),
            supports_channels: cfg!(feature = "channels"),
            supports_batching: cfg!(feature = "batching"),
            supports_compression: cfg!(feature = "compression"),
            supports_encryption: cfg!(feature = "encryption"),
            max_message_size: 1024 * 1024, // 1MB default
            max_connections: 1000,
        }
    }
}

impl Server {
    /// Create a new Server instance - just data initialization, no logic!
    pub fn new() -> Handle<Self> {
        handle(Self {
            vtable: VTable::new(),
            config: shared(ServerConfig::default()),
            stats: shared(ServerStats::default()),
            connections: shared(HashMap::new()),
            
            #[cfg(feature = "channels")]
            channels: shared(HashMap::new()),
            
            #[cfg(feature = "channels")]
            subscriptions: shared(HashMap::new()),
            
            #[cfg(feature = "batching")]
            message_queue: shared(Vec::new()),
            
            is_running: shared(false),
            capabilities: ServerCapabilities::default(),
        })
    }
}