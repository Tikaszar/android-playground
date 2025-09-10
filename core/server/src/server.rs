//! Main server contract
//!
//! This defines the contract for the main server that coordinates all components.

use std::sync::Arc;
use async_trait::async_trait;
use crate::dashboard::DashboardContract;
use crate::websocket::WebSocketContract;
use crate::channel_manager::ChannelManagerContract;
use crate::batcher::BatcherContract;
use crate::mcp::McpServerContract;
use playground_core_ecs::MessageBusContract;

/// Main server contract that implementations must fulfill
#[async_trait]
pub trait ServerContract: Send + Sync {
    /// Get dashboard component
    fn dashboard(&self) -> Arc<dyn DashboardContract>;
    
    /// Get websocket component
    fn websocket(&self) -> Arc<dyn WebSocketContract>;
    
    /// Get channel manager component
    fn channel_manager(&self) -> Arc<dyn ChannelManagerContract>;
    
    /// Get batcher component
    fn batcher(&self) -> Arc<dyn BatcherContract>;
    
    /// Get MCP server component
    fn mcp(&self) -> Arc<dyn McpServerContract>;
    
    /// Start the server on specified port
    async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Stop the server
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Connect to unified messaging system
    async fn connect_to_message_bus(&self, bus: Arc<dyn MessageBusContract>) -> Result<(), Box<dyn std::error::Error>>;
}