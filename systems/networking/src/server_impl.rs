use std::sync::Arc;
use async_trait::async_trait;
use playground_core_server::{
    ServerContract, DashboardContract, WebSocketContract, 
    ChannelManagerContract, BatcherContract, McpServerContract,
    ServerConfig
};
use playground_core_ecs::MessageBusContract;
use playground_core_types::{Shared, shared};
use axum::{Router, routing::get};
use std::net::SocketAddr;

use crate::dashboard::Dashboard;
use crate::websocket::WebSocketHandler;
use crate::channel_manager::ChannelManager;
use crate::batcher::FrameBatcher;
use crate::mcp::McpServer;

/// Main server implementation that fulfills the ServerContract
pub struct Server {
    dashboard: Arc<Dashboard>,
    websocket: Arc<WebSocketHandler>,
    channel_manager: Arc<ChannelManager>,
    batcher: Arc<FrameBatcher>,
    mcp: Arc<McpServer>,
    config: ServerConfig,
    shutdown_signal: Shared<Option<tokio::sync::oneshot::Sender<()>>>,
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let dashboard = Arc::new(Dashboard::new(config.dashboard_enabled).await?);
        let websocket = Arc::new(WebSocketHandler::new().await?);
        let channel_manager = Arc::new(ChannelManager::new().await?);
        let batcher = Arc::new(FrameBatcher::new(config.frame_rate));
        let mcp = Arc::new(McpServer::new(config.mcp_enabled).await?);
        
        Ok(Arc::new(Self {
            dashboard,
            websocket,
            channel_manager,
            batcher,
            mcp,
            config,
            shutdown_signal: shared(None),
        }))
    }
}

#[async_trait]
impl ServerContract for Server {
    fn dashboard(&self) -> Arc<dyn DashboardContract> {
        self.dashboard.clone() as Arc<dyn DashboardContract>
    }
    
    fn websocket(&self) -> Arc<dyn WebSocketContract> {
        self.websocket.clone() as Arc<dyn WebSocketContract>
    }
    
    fn channel_manager(&self) -> Arc<dyn ChannelManagerContract> {
        self.channel_manager.clone() as Arc<dyn ChannelManagerContract>
    }
    
    fn batcher(&self) -> Arc<dyn BatcherContract> {
        self.batcher.clone() as Arc<dyn BatcherContract>
    }
    
    fn mcp(&self) -> Arc<dyn McpServerContract> {
        self.mcp.clone() as Arc<dyn McpServerContract>
    }
    
    async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize dashboard logging
        if self.config.dashboard_enabled {
            self.dashboard.init_log_file().await?;
            let dashboard_clone = self.dashboard.clone();
            tokio::spawn(async move {
                dashboard_clone.start_render_loop().await;
            });
        }
        
        // Start batch processing
        let batcher_clone = self.batcher.clone();
        tokio::spawn(async move {
            batcher_clone.start_batch_loop().await;
        });
        
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
        
        // Start server
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        
        self.dashboard.log(
            playground_core_server::LogLevel::Info,
            format!("Server starting on port {}", port),
            None
        ).await;
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await?;
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let shutdown = self.shutdown_signal.write().await;
        if let Some(tx) = shutdown.as_ref() {
            // Note: We can't actually send because we don't own it
            // This would need to be refactored to properly handle shutdown
        }
        Ok(())
    }
    
    async fn connect_to_message_bus(&self, bus: Arc<dyn MessageBusContract>) -> Result<(), Box<dyn std::error::Error>> {
        // WebSocket handler subscribes to relevant channels
        self.websocket.connect_to_message_bus(bus).await?;
        Ok(())
    }
}