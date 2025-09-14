//! Generic system command processor for cross-system communication
//!
//! This allows any system to register a command processor with the World,
//! enabling other systems to communicate without direct dependencies.

use async_trait::async_trait;
use crate::{EcsResult, EcsError};
use serde::{Deserialize, Serialize};
use bytes::Bytes;

/// Generic command that can be sent to any system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCommand {
    /// Target system name
    pub target_system: String,
    /// Command type identifier
    pub command_type: String,
    /// Command payload (serialized)
    pub payload: Bytes,
}

/// Response from a system command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResponse {
    /// Success flag
    pub success: bool,
    /// Response payload (serialized)
    pub payload: Option<Bytes>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Trait for systems that can handle commands
#[async_trait]
pub trait SystemCommandProcessor: Send + Sync {
    /// Get the name of this system
    fn system_name(&self) -> &str;
    
    /// Handle a command sent to this system
    async fn handle_system_command(&self, command_type: &str, payload: Bytes) -> EcsResult<SystemResponse>;
    
    /// Get list of supported command types
    fn supported_commands(&self) -> Vec<String>;
}

/// Extended World contract for system command processors
#[async_trait]
pub trait WorldSystemCommands: Send + Sync {
    /// Register a system command processor
    async fn register_system_processor(&self, processor: std::sync::Arc<dyn SystemCommandProcessor>) -> EcsResult<()>;
    
    /// Unregister a system command processor
    async fn unregister_system_processor(&self, system_name: &str) -> EcsResult<()>;
    
    /// Send a command to a system
    async fn send_system_command(&self, command: SystemCommand) -> EcsResult<SystemResponse>;
    
    /// Get list of registered systems
    async fn registered_systems(&self) -> EcsResult<Vec<String>>;
    
    /// Check if a system is registered
    async fn has_system(&self, system_name: &str) -> bool;
}

/// Static functions for system command access through ECS
pub mod system_command_access {
    use super::*;
    use playground_core_types::{Shared, shared};
    use tokio::sync::mpsc;
    use once_cell::sync::Lazy;
    
    type CommandSender = mpsc::Sender<(SystemCommand, mpsc::Sender<EcsResult<SystemResponse>>)>;
    static COMMAND_SENDER: Lazy<Shared<Option<CommandSender>>> = Lazy::new(|| shared(None));
    
    /// Register the system command processor channel
    pub async fn register_processor(sender: CommandSender) -> EcsResult<()> {
        let mut guard = COMMAND_SENDER.write().await;
        *guard = Some(sender);
        Ok(())
    }
    
    /// Send a command to a system
    pub async fn send_to_system(target_system: &str, command_type: &str, payload: Bytes) -> EcsResult<SystemResponse> {
        let sender = {
            let guard = COMMAND_SENDER.read().await;
            guard.as_ref().ok_or(EcsError::NotInitialized)?.clone()
        };
        
        let command = SystemCommand {
            target_system: target_system.to_string(),
            command_type: command_type.to_string(),
            payload,
        };
        
        let (response_tx, mut response_rx) = mpsc::channel(1);
        sender.send((command, response_tx)).await
            .map_err(|_| EcsError::SendError)?;
        
        response_rx.recv().await
            .ok_or(EcsError::ReceiveError)?
    }
    
    /// Helper to send a command with JSON serialization
    pub async fn send_json<T: Serialize>(target_system: &str, command_type: &str, data: &T) -> EcsResult<SystemResponse> {
        let json = serde_json::to_vec(data)
            .map_err(|e| EcsError::Generic(e.to_string()))?;
        send_to_system(target_system, command_type, Bytes::from(json)).await
    }
    
    /// Helper to deserialize a JSON response
    pub fn parse_json_response<T: for<'de> Deserialize<'de>>(response: &SystemResponse) -> EcsResult<T> {
        let payload = response.payload.as_ref()
            .ok_or_else(|| EcsError::Generic("No payload in response".to_string()))?;
        
        serde_json::from_slice(payload)
            .map_err(|e| EcsError::Generic(e.to_string()))
    }
}