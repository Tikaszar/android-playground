//! MCP server contract for LLM integration
//!
//! This defines the contract for the Model Context Protocol (MCP) server.

use async_trait::async_trait;
use axum::Router;
use crate::types::{McpTool, McpRequest, McpResponse};

/// Contract for MCP (Model Context Protocol) server
#[async_trait]
pub trait McpServerContract: Send + Sync {
    /// Register an MCP tool
    async fn register_tool(&self, tool: McpTool) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Unregister an MCP tool
    async fn unregister_tool(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Handle MCP request
    async fn handle_request(&self, request: McpRequest) -> Result<McpResponse, Box<dyn std::error::Error>>;
    
    /// Get router for Axum integration
    fn router(&self) -> Router;
    
    /// List all registered tools
    async fn list_tools(&self) -> Vec<McpTool>;
}