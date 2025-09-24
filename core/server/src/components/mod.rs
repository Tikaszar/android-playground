//! Server components for ECS

pub mod connection;
pub mod server_config;
pub mod server_state;
pub mod server_stats;

#[cfg(feature = "channels")]
pub mod channel;

#[cfg(feature = "batching")]
pub mod message_queue;

// Re-export all components
pub use connection::ServerConnection;
pub use server_config::ServerConfigComponent;
pub use server_state::ServerState;
pub use server_stats::ServerStatsComponent;

#[cfg(feature = "channels")]
pub use channel::ServerChannel;

#[cfg(feature = "batching")]
pub use message_queue::MessageQueue;