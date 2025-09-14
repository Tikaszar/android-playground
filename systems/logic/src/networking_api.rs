//! Public API for networking operations
//!
//! This module provides clean functions for networking operations that internally
//! use the command processor pattern to communicate with the NetworkingSystem.

use playground_core_server::{
    ServerCommand, ServerResponse, ServerConfig, ServerStats,
    ConnectionId, ConnectionInfo, Message, ChannelId,
};
use playground_core_types::{CoreResult, CoreError};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Global channel for sending commands to the NetworkingSystem
static NETWORKING_COMMAND_SENDER: RwLock<Option<mpsc::Sender<ServerCommand>>> = RwLock::const_new(None);

/// Register the networking command channel
pub async fn register_networking_channel(sender: mpsc::Sender<ServerCommand>) -> CoreResult<()> {
    let mut guard = NETWORKING_COMMAND_SENDER.write().await;
    *guard = Some(sender);
    Ok(())
}

/// Start the server with the given configuration
pub async fn start_server(config: ServerConfig) -> CoreResult<()> {
    send_command(ServerCommand::StartServer { config }).await?;
    Ok(())
}

/// Stop the server
pub async fn stop_server() -> CoreResult<()> {
    send_command(ServerCommand::StopServer).await?;
    Ok(())
}

/// Send a message to a specific connection
pub async fn send_message(connection: ConnectionId, message: Message) -> CoreResult<()> {
    send_command(ServerCommand::SendMessage { connection, message }).await?;
    Ok(())
}

/// Broadcast a message to all connections
pub async fn broadcast_message(message: Message) -> CoreResult<()> {
    send_command(ServerCommand::BroadcastMessage { message }).await?;
    Ok(())
}

/// Register a channel with a name
pub async fn register_channel(id: ChannelId, name: String) -> CoreResult<ChannelId> {
    match send_command(ServerCommand::RegisterChannel { id, name }).await? {
        ServerResponse::ChannelRegistered(id) => Ok(id),
        _ => Err(CoreError::Generic("Unexpected response from server".to_string())),
    }
}

/// Get server statistics
pub async fn get_server_stats() -> CoreResult<ServerStats> {
    match send_command(ServerCommand::GetStats).await? {
        ServerResponse::Stats(stats) => Ok(stats),
        _ => Err(CoreError::Generic("Unexpected response from server".to_string())),
    }
}

/// Get list of active connections
pub async fn get_connections() -> CoreResult<Vec<ConnectionInfo>> {
    match send_command(ServerCommand::GetConnections).await? {
        ServerResponse::Connections(connections) => Ok(connections),
        _ => Err(CoreError::Generic("Unexpected response from server".to_string())),
    }
}

/// Internal function to send commands through the channel
async fn send_command(command: ServerCommand) -> CoreResult<ServerResponse> {
    let guard = NETWORKING_COMMAND_SENDER.read().await;
    let sender = guard.as_ref()
        .ok_or_else(|| CoreError::NotInitialized("Networking command channel not registered".to_string()))?;
    
    // In a real implementation, we'd need a response channel mechanism
    // For now, we'll just send the command and return a placeholder response
    sender.send(command).await
        .map_err(|e| CoreError::Network(e.to_string()))?;
    
    // This is a placeholder - in reality we'd wait for the response
    Ok(ServerResponse::MessageSent)
}