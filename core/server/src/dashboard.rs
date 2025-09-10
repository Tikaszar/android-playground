//! Dashboard contract for monitoring and logging
//!
//! This defines the contract for server monitoring and logging functionality.

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use crate::types::{LogLevel, ChannelType, ClientInfo};

/// Contract for server monitoring and logging
#[async_trait]
pub trait DashboardContract: Send + Sync {
    /// Log a message with specified level
    async fn log(&self, level: LogLevel, message: String, details: Option<Value>);
    
    /// Log a message for a specific component
    async fn log_component(&self, component: &str, level: LogLevel, message: String);
    
    /// Register a channel with the dashboard
    async fn register_channel(&self, id: u16, name: String, channel_type: ChannelType);
    
    /// Update client information
    async fn update_client(&self, id: usize, info: ClientInfo);
    
    /// Initialize log file
    async fn init_log_file(&self) -> Result<(), std::io::Error>;
    
    /// Start the render loop for terminal display
    async fn start_render_loop(self: Arc<Self>);
    
    /// Get recent log entries
    async fn get_recent_logs(&self, count: usize) -> Vec<String>;
}