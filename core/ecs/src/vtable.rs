//! VTable for runtime dispatch to system implementations
//! 
//! This provides a generic channel-based dispatch mechanism.
//! Other core/* packages define their specific command types.

use std::collections::HashMap;
use bytes::Bytes;
use tokio::sync::mpsc;
use playground_core_types::{Shared, shared};

/// Generic command that can be sent through the VTable
/// Other core packages will define specific command types that convert to this
pub struct VTableCommand {
    pub capability: String,
    pub operation: String,
    pub payload: Bytes,
    pub response: mpsc::Sender<VTableResponse>,
}

/// Generic response from a VTable command
pub struct VTableResponse {
    pub success: bool,
    pub payload: Option<Bytes>,
    pub error: Option<String>,
}

/// Generic VTable that stores channels by capability name
pub struct VTable {
    /// Registered capability channels
    channels: Shared<HashMap<String, mpsc::Sender<VTableCommand>>>,
}

impl VTable {
    /// Create a new empty VTable
    pub fn new() -> Self {
        Self {
            channels: shared(HashMap::new()),
        }
    }
    
    /// Register a capability channel
    pub async fn register(
        &self,
        capability: String,
        sender: mpsc::Sender<VTableCommand>
    ) -> crate::CoreResult<()> {
        let mut channels = self.channels.write().await;
        channels.insert(capability, sender);
        Ok(())
    }
    
    /// Unregister a capability
    pub async fn unregister(&self, capability: &str) -> crate::CoreResult<()> {
        let mut channels = self.channels.write().await;
        channels.remove(capability);
        Ok(())
    }
    
    /// Get a capability channel
    pub async fn get_channel(&self, capability: &str) -> Option<mpsc::Sender<VTableCommand>> {
        let channels = self.channels.read().await;
        channels.get(capability).cloned()
    }
    
    /// Check if a capability is registered
    pub async fn has_capability(&self, capability: &str) -> bool {
        let channels = self.channels.read().await;
        channels.contains_key(capability)
    }
    
    /// List all registered capabilities
    pub async fn capabilities(&self) -> Vec<String> {
        let channels = self.channels.read().await;
        channels.keys().cloned().collect()
    }
    
    /// Send a command to a capability
    pub async fn send_command(
        &self,
        capability: &str,
        operation: String,
        payload: Bytes
    ) -> crate::CoreResult<VTableResponse> {
        let sender = self.get_channel(capability).await
            .ok_or_else(|| crate::CoreError::NotRegistered(capability.to_string()))?;
        
        let (tx, mut rx) = mpsc::channel(1);
        let command = VTableCommand {
            capability: capability.to_string(),
            operation,
            payload,
            response: tx,
        };
        
        sender.send(command).await
            .map_err(|_| crate::CoreError::SendError)?;
        
        rx.recv().await
            .ok_or(crate::CoreError::ReceiveError)
    }
}