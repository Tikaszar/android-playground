//! Core server data structures and types
//!
//! This package defines ONLY data structures - NO LOGIC!
//! All server implementation logic lives in systems/networking.
//! 
//! This follows the "abstract base class" pattern where core defines
//! structure and systems provide behavior.

// Data structure modules
pub mod server;
pub mod operations;
pub mod types;
pub mod api;

// Re-export the main Server struct
pub use server::{Server, ServerCapabilities};

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

// Re-export API functions
pub use api::{
    get_server_instance,
    start_server,
    stop_server,
    is_server_running,
    get_server_stats,
    get_server_config,
    send_to_connection,
    broadcast_message,
    get_connections,
    get_connection,
    handle_connection,
    handle_disconnection,
};

#[cfg(feature = "channels")]
pub use api::{
    publish_to_channel,
    subscribe_to_channel,
    unsubscribe_from_channel,
};