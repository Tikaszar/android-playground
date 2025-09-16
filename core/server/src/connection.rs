//! Generic connection contract
//!
//! This defines a generic connection that can be implemented by any transport

use async_trait::async_trait;
use crate::types::*;
use playground_core_types::CoreError;
use std::collections::HashMap;

/// Generic contract for any connection implementation
#[async_trait]
pub trait ConnectionContract: Send + Sync {
    /// Get connection ID
    fn id(&self) -> ConnectionId;
    
    /// Get connection info
    async fn info(&self) -> ConnectionInfo;
    
    /// Send a message through this connection
    async fn send(&self, message: Message) -> Result<(), CoreError>;
    
    /// Receive a message from this connection (if available)
    async fn receive(&self) -> Result<Option<Message>, CoreError>;
    
    /// Close the connection
    async fn close(&self) -> Result<(), CoreError>;
    
    /// Check if connection is still active
    async fn is_active(&self) -> bool;
    
    /// Get connection metadata
    async fn metadata(&self) -> HashMap<String, String>;
    
    /// Set connection metadata
    async fn set_metadata(&self, key: String, value: String) -> Result<(), CoreError>;
}