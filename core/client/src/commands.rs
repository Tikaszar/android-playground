//! Command processor pattern for client operations

use crate::types::*;
use crate::input::InputEvent;
use playground_core_ecs::{EcsResult, EcsError};
use playground_core_rendering::RenderCommand;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Commands for client operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientCommand {
    /// Initialize client
    Initialize { config: ClientConfig },
    /// Connect to server
    Connect { address: String },
    /// Disconnect from server
    Disconnect,
    /// Get client state
    GetState,
    /// Send data to server
    Send { data: Vec<u8> },
    /// Receive data from server
    Receive,
    /// Update client
    Update { delta_time: f32 },
    /// Get statistics
    GetStats,
    /// Create render target
    CreateRenderTarget { target: RenderTarget },
    /// Destroy render target
    DestroyRenderTarget { id: u32 },
    /// Set render target
    SetRenderTarget { id: u32 },
    /// Submit render commands
    Render { commands: Vec<RenderCommand> },
    /// Present frame
    Present,
    /// Resize render target
    Resize { id: u32, width: u32, height: u32 },
    /// Poll input
    PollInput,
    /// Set input capture
    SetInputCapture { capture: bool },
}

/// Responses from client commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientResponse {
    /// Generic success
    Success,
    /// Client state
    State(ClientState),
    /// Data received
    Data(Option<Vec<u8>>),
    /// Statistics
    Stats(ClientStats),
    /// Render target ID
    RenderTargetId(u32),
    /// Input events
    InputEvents(Vec<InputEvent>),
    /// Error response
    Error(String),
}

/// Trait for handling client commands
#[async_trait]
pub trait ClientCommandHandler: Send + Sync {
    /// Handle a client command
    async fn handle_command(&self, command: ClientCommand) -> EcsResult<ClientResponse>;
}

/// Static functions for client access through ECS
pub mod client_access {
    use super::*;
    use playground_core_types::{Shared, shared};
    use tokio::sync::mpsc;
    use once_cell::sync::Lazy;
    
    type CommandSender = mpsc::Sender<(ClientCommand, mpsc::Sender<EcsResult<ClientResponse>>)>;
    static COMMAND_SENDER: Lazy<Shared<Option<CommandSender>>> = Lazy::new(|| shared(None));
    
    /// Register the client command processor
    pub async fn register_processor(sender: CommandSender) -> EcsResult<()> {
        let mut guard = COMMAND_SENDER.write().await;
        *guard = Some(sender);
        Ok(())
    }
    
    /// Internal helper to send commands
    async fn send_command(cmd: ClientCommand) -> EcsResult<ClientResponse> {
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
    
    /// Initialize client
    pub async fn initialize(config: ClientConfig) -> EcsResult<()> {
        send_command(ClientCommand::Initialize { config }).await?;
        Ok(())
    }
    
    /// Connect to server
    pub async fn connect(address: &str) -> EcsResult<()> {
        send_command(ClientCommand::Connect { address: address.to_string() }).await?;
        Ok(())
    }
    
    /// Disconnect from server
    pub async fn disconnect() -> EcsResult<()> {
        send_command(ClientCommand::Disconnect).await?;
        Ok(())
    }
    
    /// Get client state
    pub async fn get_state() -> EcsResult<ClientState> {
        match send_command(ClientCommand::GetState).await? {
            ClientResponse::State(state) => Ok(state),
            ClientResponse::Error(e) => Err(EcsError::Generic(e)),
            _ => Err(EcsError::Generic("Unexpected response".to_string())),
        }
    }
    
    /// Submit render commands
    pub async fn render(commands: Vec<RenderCommand>) -> EcsResult<()> {
        send_command(ClientCommand::Render { commands }).await?;
        Ok(())
    }
    
    /// Poll for input events
    pub async fn poll_input() -> EcsResult<Vec<InputEvent>> {
        match send_command(ClientCommand::PollInput).await? {
            ClientResponse::InputEvents(events) => Ok(events),
            ClientResponse::Error(e) => Err(EcsError::Generic(e)),
            _ => Err(EcsError::Generic("Unexpected response".to_string())),
        }
    }
}