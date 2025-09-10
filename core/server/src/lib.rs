//! Core server contracts and types
//!
//! This package defines ONLY contracts (traits) and types for server infrastructure.
//! All implementations live in systems/networking.

// Contract modules
pub mod server;
pub mod dashboard;
pub mod websocket;
pub mod channel_manager;
pub mod batcher;
pub mod mcp;
pub mod types;

// Re-export all contracts
pub use server::ServerContract;
pub use dashboard::DashboardContract;
pub use websocket::WebSocketContract;
pub use channel_manager::ChannelManagerContract;
pub use batcher::BatcherContract;
pub use mcp::McpServerContract;

// Re-export all types
pub use types::{
    // Packet types
    Packet,
    Priority,
    
    // Logging types
    LogLevel,
    
    // Channel types
    ChannelType,
    ChannelManifest,
    DashboardChannelInfo,
    
    // Client types
    ClientInfo,
    ClientStatus,
    ConnectionHandle,
    
    // MCP types
    McpTool,
    McpRequest,
    McpResponse,
    McpError,
    
    // Statistics and config
    NetworkStats,
    ServerConfig,
};