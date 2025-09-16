//! WebSocket/HTTP server implementation for networking system
//!
//! This file no longer contains trait implementations. All server logic
//! is now in vtable_handlers.rs and accessed via VTable dispatch.

use std::sync::Arc;
use std::time::Instant;
use playground_core_server::{ServerConfig, ServerStats};
use playground_core_types::{Shared, shared, CoreResult, CoreError};

use crate::websocket::WebSocketHandler;
use crate::channel_manager::ChannelManager;
use crate::batcher::FrameBatcher;
use crate::mcp::McpServer;
use crate::types::WebSocketConfig;

/// WebSocket/HTTP server implementation
/// This struct is now just for internal use by vtable_handlers.rs
pub struct NetworkServer {
    /// WebSocket handler
    pub websocket: Arc<WebSocketHandler>,
    /// Channel manager for logical message grouping
    pub channel_manager: Arc<ChannelManager>,
    /// Frame batcher for efficient message sending
    pub batcher: Arc<FrameBatcher>,
    /// MCP server for AI/LLM integration
    pub mcp: Arc<McpServer>,
    /// Server configuration
    pub config: Shared<ServerConfig>,
    /// WebSocket-specific configuration
    pub ws_config: WebSocketConfig,
    /// Server statistics
    pub stats: Shared<ServerStats>,
    /// Running state
    pub running: Shared<bool>,
    /// Shutdown signal
    pub shutdown_signal: Shared<Option<tokio::sync::oneshot::Sender<()>>>,
    /// Start time
    pub start_time: Instant,
}

impl NetworkServer {
    pub async fn new(ws_config: WebSocketConfig) -> CoreResult<Arc<Self>> {
        let websocket = Arc::new(WebSocketHandler::new().await
            .map_err(|e| CoreError::Generic(e.to_string()))?);
        let channel_manager = Arc::new(ChannelManager::new().await
            .map_err(|e| CoreError::Generic(e.to_string()))?);
        let batcher = Arc::new(FrameBatcher::new(ws_config.frame_rate));
        let mcp = Arc::new(McpServer::new(ws_config.mcp_enabled).await
            .map_err(|e| CoreError::Generic(e.to_string()))?);
        
        Ok(Arc::new(Self {
            websocket,
            channel_manager,
            batcher,
            mcp,
            config: shared(ServerConfig::default()),
            ws_config,
            stats: shared(ServerStats::default()),
            running: shared(false),
            shutdown_signal: shared(None),
            start_time: Instant::now(),
        }))
    }
    
    pub fn websocket(&self) -> Arc<WebSocketHandler> {
        self.websocket.clone()
    }
    
    pub fn channel_manager(&self) -> Arc<ChannelManager> {
        self.channel_manager.clone()
    }
    
    pub fn batcher(&self) -> Arc<FrameBatcher> {
        self.batcher.clone()
    }
    
    pub fn mcp(&self) -> Arc<McpServer> {
        self.mcp.clone()
    }
}