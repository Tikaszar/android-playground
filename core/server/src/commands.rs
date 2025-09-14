//! Command processor pattern for server operations

use crate::types::*;
use playground_core_ecs::{EcsResult, EcsError};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Commands for server operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerCommand {
    /// Start the server
    Start { config: ServerConfig },
    /// Stop the server
    Stop,
    /// Check if running
    IsRunning,
    /// Get statistics
    GetStats,
    /// Get configuration
    GetConfig,
    /// Send message to connection
    SendTo { connection: ConnectionId, message: Message },
    /// Broadcast message
    Broadcast { message: Message },
    /// Publish to channel
    Publish { channel: ChannelId, message: Message },
    /// Get connections
    GetConnections,
    /// Get specific connection
    GetConnection { id: ConnectionId },
    /// Create channel
    CreateChannel { name: String, description: Option<String> },
    /// Delete channel
    DeleteChannel { id: ChannelId },
    /// Subscribe to channel
    Subscribe { channel: ChannelId, connection: ConnectionId },
    /// Unsubscribe from channel
    Unsubscribe { channel: ChannelId, connection: ConnectionId },
    /// Get channel info
    GetChannelInfo { id: ChannelId },
    /// List channels
    ListChannels,
}

/// Responses from server commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerResponse {
    /// Generic success
    Success,
    /// Boolean response
    Bool(bool),
    /// Statistics response
    Stats(ServerStats),
    /// Configuration response
    Config(ServerConfig),
    /// Connection list response
    Connections(Vec<ConnectionInfo>),
    /// Single connection response
    Connection(Option<ConnectionInfo>),
    /// Channel ID response
    ChannelId(ChannelId),
    /// Channel info response
    ChannelInfo(Option<ChannelInfo>),
    /// Channel list response
    Channels(Vec<ChannelInfo>),
    /// Error response
    Error(String),
}

/// Trait for handling server commands
#[async_trait]
pub trait ServerCommandHandler: Send + Sync {
    /// Handle a server command
    async fn handle_command(&self, command: ServerCommand) -> EcsResult<ServerResponse>;
}

/// Static functions for server access through ECS
/// These will send commands through the World command processor
pub mod server_access {
    use super::*;
    use playground_core_types::{Shared, shared};
    use tokio::sync::mpsc;
    use once_cell::sync::Lazy;
    
    type CommandSender = mpsc::Sender<(ServerCommand, mpsc::Sender<EcsResult<ServerResponse>>)>;
    static COMMAND_SENDER: Lazy<Shared<Option<CommandSender>>> = Lazy::new(|| shared(None));
    
    /// Register the server command processor
    pub async fn register_processor(sender: CommandSender) -> EcsResult<()> {
        let mut guard = COMMAND_SENDER.write().await;
        *guard = Some(sender);
        Ok(())
    }
    
    /// Internal helper to send commands
    async fn send_command(cmd: ServerCommand) -> EcsResult<ServerResponse> {
        let sender = {
            let guard = COMMAND_SENDER.read().await;
            guard.as_ref().ok_or(EcsError::NotInitialized)?.clone()
        };
        
        let (response_tx, mut response_rx) = mpsc::channel(1);
        sender.send((cmd, response_tx)).await
            .map_err(|_| EcsError::SendError)?;
        
        response_rx.recv().await
            .ok_or(EcsError::ReceiveError)?
    }
    
    /// Start the server
    pub async fn start_server(config: ServerConfig) -> EcsResult<()> {
        send_command(ServerCommand::Start { config }).await?;
        Ok(())
    }
    
    /// Stop the server
    pub async fn stop_server() -> EcsResult<()> {
        send_command(ServerCommand::Stop).await?;
        Ok(())
    }
    
    /// Check if server is running
    pub async fn is_running() -> EcsResult<bool> {
        match send_command(ServerCommand::IsRunning).await? {
            ServerResponse::Bool(running) => Ok(running),
            ServerResponse::Error(e) => Err(EcsError::Generic(e)),
            _ => Err(EcsError::Generic("Unexpected response".to_string())),
        }
    }
    
    /// Send message to specific connection
    pub async fn send_to(connection: ConnectionId, message: Message) -> EcsResult<()> {
        send_command(ServerCommand::SendTo { connection, message }).await?;
        Ok(())
    }
    
    /// Broadcast message to all connections
    pub async fn broadcast(message: Message) -> EcsResult<()> {
        send_command(ServerCommand::Broadcast { message }).await?;
        Ok(())
    }
    
    /// Publish message to channel
    pub async fn publish(channel: ChannelId, message: Message) -> EcsResult<()> {
        send_command(ServerCommand::Publish { channel, message }).await?;
        Ok(())
    }
    
    /// Create a new channel
    pub async fn create_channel(name: String, description: Option<String>) -> EcsResult<ChannelId> {
        match send_command(ServerCommand::CreateChannel { name, description }).await? {
            ServerResponse::ChannelId(id) => Ok(id),
            ServerResponse::Error(e) => Err(EcsError::Generic(e)),
            _ => Err(EcsError::Generic("Unexpected response".to_string())),
        }
    }
}