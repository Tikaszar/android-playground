//! Core server contracts and types
//!
//! This package defines GENERIC contracts (traits) and types for server infrastructure.
//! These contracts can be implemented by ANY server type (WebSocket, TCP, UDP, IPC, etc.).
//! All implementations live in systems/* packages.

// Contract modules
pub mod server;
pub mod connection;
pub mod channel;
pub mod message;
pub mod commands;
pub mod types;

// Re-export all contracts
pub use server::ServerContract;
pub use connection::ConnectionContract;
pub use channel::ChannelContract;
pub use message::{MessageContract, MessageHandler};

// Re-export all types
pub use types::{
    // Generic message types
    Message,
    MessagePriority,
    MessageId,
    
    // Generic connection types
    ConnectionId,
    ConnectionInfo,
    ConnectionStatus,
    
    // Generic channel types
    ChannelId,
    ChannelInfo,
    
    // Server configuration
    ServerConfig,
    ServerStats,
};

// Re-export command types
pub use commands::{ServerCommand, ServerResponse, ServerCommandHandler};