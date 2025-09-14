//! Generic connection contract
//!
//! This defines a generic connection that can be implemented by any transport

use async_trait::async_trait;
use crate::types::*;
use std::error::Error;

/// Generic contract for any connection implementation
#[async_trait]
pub trait ConnectionContract: Send + Sync {
    /// Get connection ID
    fn id(&self) -> ConnectionId;
    
    /// Get connection info
    async fn info(&self) -> ConnectionInfo;
    
    /// Send a message through this connection
    async fn send(&self, message: Message) -> Result<(), Box<dyn Error>>;
    
    /// Receive a message from this connection (if available)
    async fn receive(&self) -> Result<Option<Message>, Box<dyn Error>>;
    
    /// Close the connection
    async fn close(&self) -> Result<(), Box<dyn Error>>;
    
    /// Check if connection is still active
    async fn is_active(&self) -> bool;
    
    /// Get connection metadata
    async fn metadata(&self) -> HashMap<String, String>;
    
    /// Set connection metadata
    async fn set_metadata(&self, key: String, value: String) -> Result<(), Box<dyn Error>>;
}

use std::collections::HashMap;