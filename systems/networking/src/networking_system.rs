use crate::{
    NetworkError, NetworkResult,
    ChannelManager, PacketQueue, IncomingPacket,
    ConnectionComponent, NetworkStatsComponent,
    NetworkStats,
};
use playground_core_ecs::{World, EntityId, Component};
use playground_core_types::{ChannelId, Priority};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main networking system that Plugins interact with
/// Now uses MessageBus for internal communication instead of WebSocket
pub struct NetworkingSystem {
    // Internal ECS world for managing network state
    world: Arc<RwLock<World>>,
    // Channel manager for dynamic registration
    channel_manager: Arc<RwLock<ChannelManager>>,
    // Packet queue for batching (kept for compatibility)
    packet_queue: Arc<RwLock<PacketQueue>>,
    // Dashboard reference (only available after server starts)
    dashboard: Option<Arc<playground_core_server::Dashboard>>,
}

impl NetworkingSystem {
    /// Create a new networking system
    pub async fn new() -> NetworkResult<Self> {
        let world = Arc::new(RwLock::new(World::new()));
        
        Ok(Self {
            world,
            channel_manager: Arc::new(RwLock::new(ChannelManager::new())),
            packet_queue: Arc::new(RwLock::new(PacketQueue::new())),
            dashboard: None,
        })
    }
    
    /// Get dashboard reference if available
    pub async fn get_dashboard(&self) -> Option<Arc<playground_core_server::Dashboard>> {
        self.dashboard.clone()
    }
    
    /// Initialize and start core/server
    pub async fn initialize(&mut self, _server_url: Option<String>) -> NetworkResult<()> {
        // Start the core server internally 
        // Note: We no longer connect via WebSocket - systems use MessageBus
        match Self::start_core_server().await {
            Ok(dashboard) => {
                self.dashboard = Some(dashboard);
            }
            Err(e) => {
                eprintln!("Core server startup failed: {}", e);
                return Err(NetworkError::ConnectionFailed(format!("Failed to start core server: {}", e)));
            }
        }
        
        // Register systems channels (1-999) locally
        let mut manager = self.channel_manager.write().await;
        let _channel_id = manager.register_system_channel("networking", 100).await?;
        
        // No WebSocket client needed - we use MessageBus for internal communication
        
        Ok(())
    }
    
    /// Register a System channel (1-999)
    pub async fn register_system_channel(&self, system_name: &str, channel_id: u16) -> NetworkResult<ChannelId> {
        if channel_id >= 1000 {
            return Err(NetworkError::ChannelError("System channels must be < 1000".to_string()));
        }
        
        let mut manager = self.channel_manager.write().await;
        manager.register_channel(channel_id, system_name.to_string()).await?;
        
        // Register with the WebSocket server if connected
        if let Some(ws) = &self.ws_client {
            ws.register_channel(channel_id, system_name).await?;
        }
        
        Ok(channel_id)
    }
    
    /// Register a Plugin for a dynamic channel (1000+)
    pub async fn register_plugin(&self, plugin_name: &str) -> NetworkResult<ChannelId> {
        let mut manager = self.channel_manager.write().await;
        let channel_id = manager.register_plugin_channel(plugin_name).await?;
        
        // Register with the WebSocket server
        if let Some(ws) = &self.ws_client {
            ws.register_channel(channel_id, plugin_name).await?;
        }
        
        Ok(channel_id)
    }
    
    /// Register an MCP tool that can be called by LLMs
    /// The tool will forward calls to the specified channel
    pub async fn register_mcp_tool(
        &self,
        name: String,
        description: String,
        input_schema: serde_json::Value,
        handler_channel: u16,
    ) -> NetworkResult<()> {
        // Note: This requires access to the WebSocketState from core/server
        // which we'll need to expose through a channel message
        
        // Send a control message to register the tool
        let registration = serde_json::json!({
            "type": "register_mcp_tool",
            "name": name,
            "description": description,
            "input_schema": input_schema,
            "handler_channel": handler_channel,
        });
        
        let data = serde_json::to_vec(&registration)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;
        
        // Send on control channel (0) with high priority
        self.send_packet(0, 100, data, Priority::High).await?;
        
        Ok(())
    }
    
    /// Unregister an MCP tool
    pub async fn unregister_mcp_tool(&self, name: &str) -> NetworkResult<()> {
        let unregistration = serde_json::json!({
            "type": "unregister_mcp_tool",
            "name": name,
        });
        
        let data = serde_json::to_vec(&unregistration)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;
        
        self.send_packet(0, 101, data, Priority::High).await?;
        
        Ok(())
    }
    
    /// Send a packet to a specific channel with priority
    pub async fn send_packet(
        &self,
        channel: ChannelId,
        packet_type: u16,
        data: Vec<u8>,
        _priority: Priority,
    ) -> NetworkResult<()> {
        // Log packet send
        if let Some(ref dashboard) = self.dashboard {
            dashboard.log(
                playground_core_server::dashboard::LogLevel::Debug,
                format!("NetworkingSystem: Publishing packet type {} on channel {} ({} bytes) to MessageBus", 
                    packet_type, channel, data.len()),
                None
            ).await;
        }
        
        // Publish to the message bus instead of WebSocket
        // The MessageBridge in core/server will forward to WebSocket clients
        use bytes::Bytes;
        self.world.read().await
            .publish(channel, Bytes::from(data.clone()))
            .await
            .map_err(|e| NetworkError::SendError(format!("Failed to publish: {:?}", e)))?;
        
        // Also queue it locally for tracking (if still needed)
        let mut queue = self.packet_queue.write().await;
        queue.enqueue(channel, packet_type, data, _priority).await
    }
    
    /// Process incoming packets for a channel
    pub async fn receive_packets(&self, channel: ChannelId) -> NetworkResult<Vec<IncomingPacket>> {
        let mut result = Vec::new();
        
        // Get packets from WebSocket
        if let Some(ws) = &self.ws_client {
            let packets = ws.receive_packets().await;
            
            // Filter for requested channel and convert to IncomingPacket
            for packet in packets {
                if packet.channel_id == channel {
                    result.push(IncomingPacket {
                        channel,
                        packet_type: packet.packet_type,
                        data: packet.payload.to_vec(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    });
                }
            }
        }
        
        // Also check local queue
        let queue = self.packet_queue.read().await;
        let local_packets = queue.get_incoming(channel).await?;
        result.extend(local_packets);
        
        Ok(result)
    }
    
    /// Create a peer-to-peer connection entity
    pub async fn create_connection(&self, peer_id: String) -> NetworkResult<EntityId> {
        let world = self.world.write().await;
        
        // Register the component type if not already registered
        world.register_component::<ConnectionComponent>().await
            .map_err(|e| NetworkError::EcsError(e.to_string()))?;
        
        // Create the connection component
        let connection = ConnectionComponent {
            peer_id: peer_id.clone(),
            connected: false,
            latency_ms: 0,
            packets_sent: 0,
            packets_received: 0,
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };
        
        // Spawn entity with the component
        let components: Vec<Box<dyn playground_core_ecs::Component>> = vec![
            Box::new(connection),
        ];
        
        let entities = world.spawn_batch(vec![components]).await
            .map_err(|e| NetworkError::EcsError(e.to_string()))?;
        
        Ok(entities[0])
    }
    
    /// Query connection components
    pub async fn query_connections(&self) -> NetworkResult<Vec<ConnectionComponent>> {
        let world = self.world.read().await;
        
        // Build and execute query  
        let query = world.query().with_component(ConnectionComponent::component_id()).build();
        let entity_ids = world.execute_query(query.as_ref()).await
            .map_err(|e| NetworkError::EcsError(e.to_string()))?;
        
        // Extract connection components
        let mut connections = Vec::new();
        for entity in entity_ids {
            if let Ok(component) = world.get_component::<ConnectionComponent>(entity).await {
                connections.push(component);
            }
        }
        
        Ok(connections)
    }
    
    /// Send a reliable packet (with retries and acknowledgment)
    pub async fn send_reliable(
        &self,
        channel: ChannelId,
        packet_type: u16,
        data: Vec<u8>,
    ) -> NetworkResult<()> {
        // For now, just send with Critical priority
        // TODO: Implement actual reliability with acks
        self.send_packet(channel, packet_type, data, Priority::Critical).await
    }
    
    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkResult<NetworkStats> {
        let world = self.world.read().await;
        
        // Query all NetworkStatsComponents
        let stats_query = world.query().with_component(NetworkStatsComponent::component_id()).build();
        let entity_ids = world.execute_query(stats_query.as_ref()).await
            .map_err(|e| NetworkError::EcsError(e.to_string()))?;
        
        // Aggregate stats
        let mut total_stats = NetworkStats {
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            connections_active: 0,
            average_latency_ms: 0,
        };
        
        let mut total_latency = 0u64;
        let mut latency_count = 0u64;
        
        for entity in entity_ids {
            if let Ok(stats) = world.get_component::<NetworkStatsComponent>(entity).await {
                total_stats.bytes_sent += stats.bytes_sent;
                total_stats.bytes_received += stats.bytes_received;
                total_stats.packets_sent += stats.packets_sent;
                total_stats.packets_received += stats.packets_received;
                
                if stats.average_latency_ms > 0 {
                    total_latency += stats.average_latency_ms as u64;
                    latency_count += 1;
                }
            }
        }
        
        // Count active connections
        let conn_query = world.query().with_component(ConnectionComponent::component_id()).build();
        let connections = world.execute_query(conn_query.as_ref()).await
            .map_err(|e| NetworkError::EcsError(e.to_string()))?;
        
        let mut active_count = 0;
        for entity in connections {
            if let Ok(conn) = world.get_component::<ConnectionComponent>(entity).await {
                if conn.connected {
                    active_count += 1;
                }
            }
        }
        total_stats.connections_active = active_count;
        
        // Calculate average latency
        if latency_count > 0 {
            total_stats.average_latency_ms = (total_latency / latency_count) as u32;
        }
        
        Ok(total_stats)
    }
    
    /// Start the core server internally (called by initialize)
    /// This version starts the server and returns immediately with the dashboard reference
    async fn start_core_server() -> Result<Arc<playground_core_server::Dashboard>, Box<dyn std::error::Error>> {
        use playground_core_server::{
            Dashboard, McpServer, WebSocketState, websocket_handler,
            list_plugins, reload_plugin, root,
            dashboard::LogLevel,
        };
        use axum::{Router, routing::{get, post}};
        use std::net::SocketAddr;
        use tower_http::cors::CorsLayer;
        use tower_http::services::ServeDir;
        use tower_http::trace::TraceLayer;
        
        // Server creates and owns the dashboard
        let dashboard = Arc::new(Dashboard::new());
        
        // Initialize log file
        if let Err(e) = dashboard.init_log_file().await {
            eprintln!("Failed to initialize log file: {}", e);
        }
        
        // Start dashboard render loop
        dashboard.clone().start_render_loop().await;
        
        // Create WebSocketState with the dashboard
        let ws_state = Arc::new(WebSocketState::new_with_dashboard(dashboard.clone()));
        
        // Create MCP server
        let mcp_server = McpServer::new();
        let mcp_router = mcp_server.router();
        
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        
        // Log to dashboard
        dashboard.log(
            LogLevel::Info,
            format!("Core server listening on {}", addr),
            None
        ).await;
        dashboard.log(
            LogLevel::Info,
            format!("WebSocket endpoint: ws://localhost:8080/ws"),
            None
        ).await;
        dashboard.log(
            LogLevel::Info,
            format!("MCP endpoint: http://localhost:8080/mcp"),
            None
        ).await;
        
        let app = Router::new()
            .route("/", get(root))
            .route("/ws", get(websocket_handler))
            .route("/api/plugins", get(list_plugins))
            .route("/api/reload", post(reload_plugin))
            .nest_service("/playground-editor", ServeDir::new("apps/playground-editor/static"))
            .nest("/mcp", mcp_router)
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(ws_state);
        
        // Return the dashboard reference before starting the server
        let dashboard_clone = dashboard.clone();
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        // Start server in background - MUST spawn or it blocks
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Server error: {}", e);
            }
        });
        
        // Give server a moment to fully start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(dashboard_clone)
    }
}