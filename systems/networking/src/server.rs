//! WebSocket/HTTP server implementation for networking system
//!
//! This file no longer contains trait implementations. All server logic
//! is now in vtable_handlers.rs and accessed via VTable dispatch.

use std::time::Instant;
use playground_core_server::{ServerConfig, ServerStats};
use playground_core_types::{Handle, handle, Shared, shared, CoreResult, CoreError};

use crate::websocket::WebSocketHandler;
use crate::channel_manager::ChannelManager;
use crate::batcher::FrameBatcher;
use crate::mcp::McpServer;
use crate::types::WebSocketConfig;

/// WebSocket/HTTP server implementation
/// This struct is now just for internal use by vtable_handlers.rs
pub struct NetworkServer {
    /// WebSocket handler
    pub websocket: Handle<WebSocketHandler>,
    /// Channel manager for logical message grouping
    pub channel_manager: Handle<ChannelManager>,
    /// Frame batcher for efficient message sending
    pub batcher: Handle<FrameBatcher>,
    /// MCP server for AI/LLM integration
    pub mcp: Handle<McpServer>,
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
    pub async fn new(ws_config: WebSocketConfig) -> CoreResult<Handle<Self>> {
        let websocket = handle(WebSocketHandler::new().await
            .map_err(|e| CoreError::Generic(e.to_string()))?);
        let channel_manager = handle(ChannelManager::new().await
            .map_err(|e| CoreError::Generic(e.to_string()))?);
        let batcher = handle(FrameBatcher::new(ws_config.frame_rate));
        let mcp = handle(McpServer::new(ws_config.mcp_enabled).await
            .map_err(|e| CoreError::Generic(e.to_string()))?);
        
        Ok(handle(Self {
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
    
    pub fn websocket(&self) -> Handle<WebSocketHandler> {
        self.websocket.clone()
    }

    pub fn channel_manager(&self) -> Handle<ChannelManager> {
        self.channel_manager.clone()
    }

    pub fn batcher(&self) -> Handle<FrameBatcher> {
        self.batcher.clone()
    }

    pub fn mcp(&self) -> Handle<McpServer> {
        self.mcp.clone()
    }
}