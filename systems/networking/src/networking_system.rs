use std::sync::Arc;
use async_trait::async_trait;
use playground_core_server::{
    ServerContract, ServerConfig, ServerCommand, ServerResponse, ServerCommandHandler
};
// Console imports will be added when console system is created
use playground_core_types::LogLevel as CoreLogLevel;
use playground_core_ecs::{MessageBusContract, System, ExecutionStage, EcsResult, EcsError};
use playground_core_types::{CoreResult, CoreError};
use tokio::sync::mpsc;
use crate::server_impl::Server;
use crate::types::{WebSocketConfig, McpTool, Packet, LogLevel};

/// High-level networking system that wraps the server implementation
pub struct NetworkingSystem {
    server: Option<Arc<Server>>,
    config: ServerConfig,
    ws_config: WebSocketConfig,
    next_plugin_channel: u16,
    // console_sender: Option<mpsc::Sender<ConsoleCommand>>,
}

impl NetworkingSystem {
    pub async fn new() -> CoreResult<Self> {
        Ok(Self {
            server: None,
            config: ServerConfig::default(),
            ws_config: WebSocketConfig::default(),
            next_plugin_channel: 1000, // Plugins start at 1000
            // console_sender: None,
        })
    }
    
    /// Register a plugin and get its channel ID
    pub async fn register_plugin(&mut self, name: &str) -> CoreResult<u16> {
        let channel = self.next_plugin_channel;
        self.next_plugin_channel += 1;
        
        if let Some(ref server) = self.server {
            server.channel_manager().register(channel, name.to_string()).await?;
            self.log(
                CoreLogLevel::Info,
                format!("Registered plugin '{}' on channel {}", name, channel)
            ).await;
        }
        
        Ok(channel)
    }
    
    /// Register an MCP tool
    pub async fn register_mcp_tool(&self, tool: McpTool) -> CoreResult<()> {
        if let Some(ref server) = self.server {
            server.mcp().register_tool(tool).await?;
        }
        Ok(())
    }
    
    /// Send a packet
    pub async fn send_packet(&self, packet: Packet) -> CoreResult<()> {
        if let Some(ref server) = self.server {
            server.batcher().queue_packet(packet).await;
        }
        Ok(())
    }
    
    /// Log a message (will use console command processor when available)
    async fn log(&self, _level: CoreLogLevel, message: String) {
        // For now, just print to stdout
        println!("{}", message);
    }
    
    /// Log a component message
    pub async fn log_component(&self, component: &str, level: LogLevel, message: String) {
        let core_level = match level {
            LogLevel::Trace => CoreLogLevel::Trace,
            LogLevel::Debug => CoreLogLevel::Debug,
            LogLevel::Info => CoreLogLevel::Info,
            LogLevel::Warn => CoreLogLevel::Warn,
            LogLevel::Error => CoreLogLevel::Error,
        };
        
        // Will use console command processor when available
        println!("[{}] {}: {}", component, core_level as u8, message);
    }
    
    /// Get the server (for internal use)
    pub fn server(&self) -> Option<Arc<Server>> {
        self.server.clone()
    }
    
    /// Connect to the unified message bus
    pub async fn connect_to_message_bus(&mut self, bus: Arc<dyn MessageBusContract>) -> CoreResult<()> {
        if let Some(ref mut server) = self.server {
            // Need to get mutable reference to server
            // This is a limitation - we'd need to restructure to use Arc<Mutex<>> or similar
            // For now, we'll skip this functionality
        }
        Ok(())
    }
    
    // pub fn set_console_sender(&mut self, sender: mpsc::Sender<ConsoleCommand>) {
    //     self.console_sender = Some(sender);
    //     if let Some(ref mut server) = self.server {
    //         // Same issue - need mutable server
    //     }
    // }
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
        let server = Server::new(self.ws_config.clone()).await
            .map_err(|e| EcsError::Generic(e.to_string()))?;
        self.server = Some(server.clone());
        
        // Start server in background
        let server_clone = server.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            if let Err(e) = server_clone.start(config).await {
                eprintln!("Server error: {}", e);
            }
        });
        
        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        self.log(
            CoreLogLevel::Info,
            format!("NetworkingSystem initialized on port {}", self.ws_config.port)
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
                .map_err(|e| EcsError::Generic(e.to_string()))?;
        }
        Ok(())
    }
}

#[async_trait]
impl ServerCommandHandler for NetworkingSystem {
    async fn handle_command(&self, command: ServerCommand) -> EcsResult<ServerResponse> {
        match command {
            ServerCommand::Start { config } => {
                if let Some(ref server) = self.server {
                    server.start(config).await
                        .map_err(|e| EcsError::Generic(e.to_string()))?;
                    Ok(ServerResponse::Success)
                } else {
                    Err(EcsError::Generic("Server not initialized".to_string()))
                }
            }
            ServerCommand::Stop => {
                if let Some(ref server) = self.server {
                    server.stop().await
                        .map_err(|e| EcsError::Generic(e.to_string()))?;
                    Ok(ServerResponse::Success)
                } else {
                    Err(EcsError::Generic("Server not initialized".to_string()))
                }
            }
            ServerCommand::SendTo { connection, message } => {
                if let Some(ref server) = self.server {
                    server.send_to(connection, message).await
                        .map_err(|e| EcsError::Generic(e.to_string()))?;
                    Ok(ServerResponse::Success)
                } else {
                    Err(EcsError::Generic("Server not initialized".to_string()))
                }
            }
            ServerCommand::Broadcast { message } => {
                if let Some(ref server) = self.server {
                    server.broadcast(message).await
                        .map_err(|e| EcsError::Generic(e.to_string()))?;
                    Ok(ServerResponse::Success)
                } else {
                    Err(EcsError::Generic("Server not initialized".to_string()))
                }
            }
            ServerCommand::CreateChannel { name, description } => {
                if let Some(ref server) = self.server {
                    // For now, just use a hardcoded channel ID
                    let id = playground_core_server::ChannelId(self.next_plugin_channel);
                    server.channel_manager().register(id.0, name).await
                        .map_err(|e| EcsError::Generic(e.to_string()))?;
                    Ok(ServerResponse::ChannelId(id))
                } else {
                    Err(EcsError::Generic("Server not initialized".to_string()))
                }
            }
            ServerCommand::GetStats => {
                if let Some(ref server) = self.server {
                    let stats = server.stats().await;
                    Ok(ServerResponse::Stats(stats))
                } else {
                    Err(EcsError::Generic("Server not initialized".to_string()))
                }
            }
            ServerCommand::GetConnections => {
                if let Some(ref server) = self.server {
                    let connections = server.connections().await;
                    Ok(ServerResponse::Connections(connections))
                } else {
                    Err(EcsError::Generic("Server not initialized".to_string()))
                }
            }
            _ => {
                // Other commands not implemented yet
                Err(EcsError::Generic("Command not implemented".to_string()))
            }
        }
    }
}