//! Networking System
//! 
//! This system provides networking functionality for Plugins and Apps.
//! Internally uses core/ecs for state management and integrates with
//! core/server for WebSocket communication.

mod components;
mod connection;
mod packet_queue;
mod channel_manager;
mod network_system;
mod websocket_client;
mod networking_system;
mod types;
mod register;

pub use components::*;
pub use connection::*;
pub use packet_queue::*;
pub use channel_manager::*;
pub use network_system::*;
pub use networking_system::*;
pub use types::*;
pub use register::register;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Invalid network message: {0}")]
    InvalidMessage(String),
    #[error("Network timeout: {0}")]
    Timeout(String),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("ECS error: {0}")]
    EcsError(String),
    #[error("Not connected")]
    NotConnected,
    #[error("Channel not found: {0}")]
    ChannelNotFound(u16),
    #[error("Send error: {0}")]
    SendError(String),
}

pub type NetworkResult<T> = Result<T, NetworkError>;

// Re-export WebSocketClient for internal use only
use websocket_client::WebSocketClient;