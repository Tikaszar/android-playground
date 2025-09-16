//! Public API functions for server operations
//! 
//! These functions provide a convenient way to access server functionality
//! without needing to manage the server instance directly.

use once_cell::sync::Lazy;
use playground_core_types::{Handle, CoreResult};
use crate::{Server, ServerConfig, ServerStats, ConnectionId, ConnectionInfo, Message, ChannelId};

/// Global server instance
static SERVER_INSTANCE: Lazy<Handle<Server>> = Lazy::new(|| Server::new());

/// Get the global server instance
pub fn get_server_instance() -> CoreResult<&'static Handle<Server>> {
    Ok(&*SERVER_INSTANCE)
}

/// Start the server with the given configuration
pub async fn start_server(config: ServerConfig) -> CoreResult<()> {
    get_server_instance()?.start(config).await
}

/// Stop the server
pub async fn stop_server() -> CoreResult<()> {
    get_server_instance()?.stop().await
}

/// Check if the server is running
pub async fn is_server_running() -> CoreResult<bool> {
    Ok(get_server_instance()?.is_running().await)
}

/// Get server statistics
pub async fn get_server_stats() -> CoreResult<ServerStats> {
    Ok(get_server_instance()?.stats().await)
}

/// Get server configuration
pub async fn get_server_config() -> CoreResult<ServerConfig> {
    Ok(get_server_instance()?.config().await)
}

/// Send a message to a specific connection
pub async fn send_to_connection(connection: ConnectionId, message: Message) -> CoreResult<()> {
    get_server_instance()?.send_to(connection, message).await
}

/// Broadcast a message to all connections
pub async fn broadcast_message(message: Message) -> CoreResult<()> {
    get_server_instance()?.broadcast(message).await
}

/// Publish a message to a channel
#[cfg(feature = "channels")]
pub async fn publish_to_channel(channel: ChannelId, message: Message) -> CoreResult<()> {
    get_server_instance()?.publish(channel, message).await
}

/// Subscribe a connection to a channel
#[cfg(feature = "channels")]
pub async fn subscribe_to_channel(connection: ConnectionId, channel: ChannelId) -> CoreResult<()> {
    get_server_instance()?.subscribe(connection, channel).await
}

/// Unsubscribe a connection from a channel
#[cfg(feature = "channels")]
pub async fn unsubscribe_from_channel(connection: ConnectionId, channel: ChannelId) -> CoreResult<()> {
    get_server_instance()?.unsubscribe(connection, channel).await
}

/// Get list of active connections
pub async fn get_connections() -> CoreResult<Vec<ConnectionInfo>> {
    Ok(get_server_instance()?.connections().await)
}

/// Get connection info
pub async fn get_connection(id: ConnectionId) -> CoreResult<Option<ConnectionInfo>> {
    Ok(get_server_instance()?.connection(id).await)
}

/// Handle incoming connection (called by implementation)
pub async fn handle_connection(connection: ConnectionInfo) -> CoreResult<()> {
    get_server_instance()?.on_connection(connection).await
}

/// Handle connection closed (called by implementation)
pub async fn handle_disconnection(id: ConnectionId) -> CoreResult<()> {
    get_server_instance()?.on_disconnection(id).await
}