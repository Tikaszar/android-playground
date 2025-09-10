use std::sync::Arc;
use async_trait::async_trait;
use playground_core_server::{
    ServerContract, ServerConfig, LogLevel, McpTool, Packet
};
use playground_core_ecs::{MessageBusContract, System, ExecutionStage, EcsResult};
use crate::server_impl::Server;

/// High-level networking system that wraps the server implementation
pub struct NetworkingSystem {
    server: Option<Arc<Server>>,
    config: ServerConfig,
    next_plugin_channel: u16,
}

impl NetworkingSystem {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            server: None,
            config: ServerConfig {
                port: 8080,
                dashboard_enabled: true,
                mcp_enabled: true,
                frame_rate: 60,
                max_connections: 100,
                log_to_file: true,
            },
            next_plugin_channel: 1000, // Plugins start at 1000
        })
    }
    
    /// Register a plugin and get its channel ID
    pub async fn register_plugin(&mut self, name: &str) -> Result<u16, Box<dyn std::error::Error>> {
        let channel = self.next_plugin_channel;
        self.next_plugin_channel += 1;
        
        if let Some(ref server) = self.server {
            server.channel_manager().register(channel, name.to_string()).await?;
            server.dashboard().log(
                LogLevel::Info,
                format!("Registered plugin '{}' on channel {}", name, channel),
                None
            ).await;
        }
        
        Ok(channel)
    }
    
    /// Register an MCP tool
    pub async fn register_mcp_tool(&self, tool: McpTool) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref server) = self.server {
            server.mcp().register_tool(tool).await?;
        }
        Ok(())
    }
    
    /// Send a packet
    pub async fn send_packet(&self, packet: Packet) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref server) = self.server {
            server.batcher().queue_packet(packet).await;
        }
        Ok(())
    }
    
    /// Log a message
    pub async fn log(&self, level: LogLevel, message: String) {
        if let Some(ref server) = self.server {
            server.dashboard().log(level, message, None).await;
        }
    }
    
    /// Log a component message
    pub async fn log_component(&self, component: &str, level: LogLevel, message: String) {
        if let Some(ref server) = self.server {
            server.dashboard().log_component(component, level, message).await;
        }
    }
    
    /// Get the server (for internal use)
    pub fn server(&self) -> Option<Arc<dyn ServerContract>> {
        self.server.as_ref().map(|s| s.clone() as Arc<dyn ServerContract>)
    }
    
    /// Connect to the unified message bus
    pub async fn connect_to_message_bus(&self, bus: Arc<dyn MessageBusContract>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref server) = self.server {
            server.connect_to_message_bus(bus).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl System for NetworkingSystem {
    fn name(&self) -> &str {
        "NetworkingSystem"
    }
    
    fn stage(&self) -> ExecutionStage {
        // Networking runs in Update stage
        ExecutionStage::Update
    }
    
    async fn initialize(&mut self) -> EcsResult<()> {
        // Create and start the server
        let server = Server::new(self.config.clone()).await
            .map_err(|e| playground_core_ecs::EcsError::Generic(e.to_string()))?;
        self.server = Some(server.clone());
        
        // Start server in background
        let server_clone = server.clone();
        let port = self.config.port;
        tokio::spawn(async move {
            if let Err(e) = server_clone.start(port).await {
                eprintln!("Server error: {}", e);
            }
        });
        
        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        self.log(
            LogLevel::Info,
            format!("NetworkingSystem initialized on port {}", self.config.port)
        ).await;
        
        Ok(())
    }
    
    async fn update(&mut self, _delta_time: f32) -> EcsResult<()> {
        // Networking system doesn't need regular updates
        // Everything is event-driven
        Ok(())
    }
    
    async fn cleanup(&mut self) -> EcsResult<()> {
        if let Some(ref server) = self.server {
            server.stop().await
                .map_err(|e| playground_core_ecs::EcsError::Generic(e.to_string()))?;
        }
        Ok(())
    }
}