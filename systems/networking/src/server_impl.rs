//! WebSocket/HTTP implementation of the generic server contract

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::collections::HashMap;
use async_trait::async_trait;
use playground_core_server::{
    ServerContract, ServerConfig, ServerStats,
    ConnectionId, ConnectionInfo, ConnectionStatus,
    ChannelId, Message, MessageId, MessagePriority,
};
use playground_core_types::{Shared, shared, CoreResult, CoreError};
// Console imports will be added when console system is created
// For now, we'll use a simplified approach
use playground_core_ecs::MessageBusContract;
use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::websocket::WebSocketHandler;
use crate::channel_manager::ChannelManager;
use crate::batcher::FrameBatcher;
use crate::mcp::McpServer;
use crate::types::{WebSocketConfig, ClientInfo, ConnectionHandle};

/// WebSocket/HTTP server implementation
pub struct Server {
    /// WebSocket handler
    websocket: Arc<WebSocketHandler>,
    /// Channel manager for logical message grouping
    channel_manager: Arc<ChannelManager>,
    /// Frame batcher for efficient message sending
    batcher: Arc<FrameBatcher>,
    /// MCP server for AI/LLM integration
    mcp: Arc<McpServer>,
    /// Server configuration
    config: Shared<ServerConfig>,
    /// WebSocket-specific configuration
    ws_config: WebSocketConfig,
    /// Server statistics
    stats: Shared<ServerStats>,
    /// Running state
    running: Shared<bool>,
    /// Shutdown signal
    shutdown_signal: Shared<Option<tokio::sync::oneshot::Sender<()>>>,
    /// Start time
    start_time: Instant,
    /// Console command sender (for logging) - will be added when console system is created
    // console_sender: Option<mpsc::Sender<ConsoleCommand>>,
    /// Message bus for ECS integration
    message_bus: Option<Arc<dyn MessageBusContract>>,
}

impl Server {
    pub async fn new(ws_config: WebSocketConfig) -> CoreResult<Arc<Self>> {
        let websocket = Arc::new(WebSocketHandler::new().await
            .map_err(|e| CoreError::Generic(e.to_string()))?);
        let channel_manager = Arc::new(ChannelManager::new().await
            .map_err(|e| CoreError::Generic(e.to_string()))?);
        let batcher = Arc::new(FrameBatcher::new(ws_config.frame_rate));
        let mcp = Arc::new(McpServer::new(ws_config.mcp_enabled).await
            .map_err(|e| CoreError::Generic(e.to_string()))?);
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CoreError::Generic(e.to_string()))?
            .as_secs();
        
        Ok(Arc::new(Self {
            websocket,
            channel_manager,
            batcher,
            mcp,
            config: shared(ServerConfig::default()),
            ws_config,
            stats: shared(ServerStats {
                start_time: now,
                total_connections: 0,
                active_connections: 0,
                total_messages_sent: 0,
                total_messages_received: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
                errors: 0,
            }),
            running: shared(false),
            shutdown_signal: shared(None),
            start_time: Instant::now(),
            // console_sender: None,
            message_bus: None,
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
    
    pub async fn connect_to_message_bus(&mut self, bus: Arc<dyn MessageBusContract>) -> CoreResult<()> {
        self.message_bus = Some(bus.clone());
        self.websocket.connect_to_message_bus(bus).await
            .map_err(|e| CoreError::Generic(e.to_string()))?;
        Ok(())
    }
    
    // pub fn set_console_sender(&mut self, sender: mpsc::Sender<ConsoleCommand>) {
    //     self.console_sender = Some(sender);
    // }
    
    async fn log(&self, _level: playground_core_types::LogLevel, message: String) {
        // Will use console command processor when available
        println!("{}", message);
    }
}

#[async_trait]
impl ServerContract for Server {
    async fn start(&self, config: ServerConfig) -> CoreResult<()> {
        // Update config
        {
            let mut cfg = self.config.write().await;
            *cfg = config.clone();
        }
        
        // Use WebSocket-specific port
        let port = self.ws_config.port;
        
        // Start batch processing
        if config.enable_batching {
            let batcher_clone = self.batcher.clone();
            tokio::spawn(async move {
                batcher_clone.start_batch_loop().await;
            });
        }
        
        // Create Axum app  
        let ws_routes = Router::new()
            .route("/ws", get(crate::websocket::websocket_handler))
            .with_state(self.websocket.clone());
            
        let app = Router::new()
            .route("/", get(|| async { "Playground Server" }))
            .merge(ws_routes)
            .nest("/mcp", self.mcp.router());
        
        // Create shutdown channel
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut shutdown = self.shutdown_signal.write().await;
            *shutdown = Some(tx);
        }
        
        // Create TCP listener
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await.map_err(CoreError::from)?;
        
        self.log(
            playground_core_types::LogLevel::Info,
            format!("🚀 Server listening on http://0.0.0.0:{}", port)
        ).await;
        
        // Mark as running
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // Run server
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;
        
        Ok(())
    }
    
    async fn stop(&self) -> CoreResult<()> {
        // Send shutdown signal
        let shutdown = {
            let mut shutdown = self.shutdown_signal.write().await;
            shutdown.take()
        };
        
        if let Some(tx) = shutdown {
            let _ = tx.send(());
        }
        
        // Mark as not running
        {
            let mut running = self.running.write().await;
            *running = false;
        }
        
        Ok(())
    }
    
    async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    async fn stats(&self) -> ServerStats {
        self.stats.read().await.clone()
    }
    
    async fn config(&self) -> ServerConfig {
        self.config.read().await.clone()
    }
    
    async fn on_connection(&self, connection: ConnectionInfo) -> CoreResult<()> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_connections += 1;
            stats.active_connections += 1;
        }
        
        // Store connection info
        self.websocket.store_connection(connection).await;
        
        Ok(())
    }
    
    async fn on_disconnection(&self, id: ConnectionId) -> CoreResult<()> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
        }
        
        // Remove connection info
        self.websocket.remove_connection_by_core_id(id).await;
        
        Ok(())
    }
    
    async fn send_to(&self, connection: ConnectionId, message: Message) -> CoreResult<()> {
        // Queue message in batcher
        self.batcher.queue_message(connection, message).await;
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_messages_sent += 1;
        }
        
        Ok(())
    }
    
    async fn broadcast(&self, message: Message) -> CoreResult<()> {
        // Get all connections
        let connections = self.websocket.get_all_connections().await;
        
        // Send to all
        for conn in connections {
            self.send_to(conn.id, message.clone()).await?;
        }
        
        Ok(())
    }
    
    async fn publish(&self, channel: ChannelId, message: Message) -> CoreResult<()> {
        // Get subscribers to this channel
        let subscribers = self.channel_manager.get_subscribers(channel.0).await;
        
        // Send to all subscribers
        for conn_id in subscribers {
            self.send_to(conn_id, message.clone()).await?;
        }
        
        Ok(())
    }
    
    async fn connections(&self) -> Vec<ConnectionInfo> {
        self.websocket.get_all_connections().await
    }
    
    async fn connection(&self, id: ConnectionId) -> Option<ConnectionInfo> {
        self.websocket.get_connection(id).await
    }
}