pub mod types;
pub mod state;
pub mod server;
pub mod websocket;
pub mod channel_manager;
pub mod batcher;
pub mod mcp;
pub mod networking_system;
pub mod vtable_handlers;
pub mod registration;

// Main export is the NetworkingSystem
pub use networking_system::NetworkingSystem;
// Re-export commonly used types
pub use types::{Packet, Priority, ClientInfo, ClientStatus, ChannelManifest, McpTool, LogLevel};
// Export registration for system initialization
pub use registration::initialize;