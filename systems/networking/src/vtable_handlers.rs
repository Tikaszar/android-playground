//! VTable handlers for server and client operations
//!
//! This module implements all the actual networking logic that core/server
//! and core/client delegate to via VTable. It works with ECS entities
//! and components, managing the actual network implementation internally.

use bytes::Bytes;
use playground_core_types::{Handle, handle, Shared, shared, CoreResult};
use playground_core_ecs::{VTableResponse, Entity, get_world};
use playground_core_server::{
    ServerConfig, ServerStats, ConnectionId, ConnectionInfo, ConnectionStatus,
    Message, ChannelId,
    components::*,
    api as server_api,
};
use playground_core_client::{
    ClientConfig, ClientState, ClientStats,
    components::*,
    api as client_api,
};
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use std::net::SocketAddr;

use crate::server::NetworkServer;
use crate::types::WebSocketConfig;
use crate::websocket::WebSocketHandler;
use crate::channel_manager::ChannelManager;
use crate::batcher::FrameBatcher;
use crate::mcp::McpServer;
use crate::state::NETWORK_STATE;

// Helper functions for VTableResponse
fn error_response(msg: String) -> VTableResponse {
    VTableResponse {
        success: false,
        payload: None,
        error: Some(msg),
    }
}

fn success_response(payload: Option<Bytes>) -> VTableResponse {
    VTableResponse {
        success: true,
        payload,
        error: None,
    }
}

/// Initialize network handlers (called during system registration)
pub async fn initialize() -> CoreResult<()> {
    crate::state::NetworkState::initialize().await;
    Ok(())
}

/// Handle server operations (start, stop, send_to, broadcast, publish)
pub async fn handle_server_operations(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "start" => handle_server_start(payload).await,
        "stop" => handle_server_stop().await,
        "send_to" => handle_send_to(payload).await,
        "broadcast" => handle_broadcast(payload).await,
        #[cfg(feature = "channels")]
        "publish" => handle_publish(payload).await,
        _ => error_response(format!("Unknown server operation: {}", operation)),
    }
}

/// Handle channel operations (subscribe, unsubscribe)
#[cfg(feature = "channels")]
pub async fn handle_channel_operations(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "subscribe" => handle_subscribe(payload).await,
        "unsubscribe" => handle_unsubscribe(payload).await,
        _ => error_response(format!("Unknown channel operation: {}", operation)),
    }
}

/// Handle server events (on_connection, on_disconnection)
pub async fn handle_server_events(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "on_connection" => handle_on_connection(payload).await,
        "on_disconnection" => handle_on_disconnection(payload).await,
        _ => error_response(format!("Unknown server event: {}", operation)),
    }
}

/// Handle client operations (initialize, connect, disconnect, send, receive, update)
pub async fn handle_client_operations(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "initialize" => handle_client_initialize(payload).await,
        "connect" => handle_client_connect(payload).await,
        "disconnect" => handle_client_disconnect().await,
        "send" => handle_client_send(payload).await,
        "receive" => handle_client_receive().await,
        "update" => handle_client_update(payload).await,
        _ => error_response(format!("Unknown client operation: {}", operation)),
    }
}

// Server operation implementations

async fn handle_server_start(payload: Bytes) -> VTableResponse {
    let config: ServerConfig = match bincode::deserialize(&payload) {
        Ok(c) => c,
        Err(e) => return error_response(format!("Failed to deserialize config: {}", e)),
    };

    // Create server entity with components
    let server_entity = match server_api::start_server(config.clone()).await {
        Ok(entity) => entity,
        Err(e) => return error_response(format!("Failed to create server entity: {}", e)),
    };

    // Store the server entity
    *NETWORK_STATE.server_entity.write().await = Some(server_entity.clone());

    // Create WebSocket configuration with defaults
    let ws_config = WebSocketConfig {
        port: 8080, // Default port, since ServerConfig doesn't have one
        frame_rate: 60,
        max_connections: config.max_connections,
        max_message_size: config.max_message_size,
        mcp_enabled: true,
    };

    // Create actual network server components
    let websocket = handle(match WebSocketHandler::new().await {
        Ok(ws) => ws,
        Err(e) => return error_response(format!("Failed to create WebSocket handler: {}", e)),
    });

    let channel_manager = handle(match ChannelManager::new().await {
        Ok(cm) => cm,
        Err(e) => return error_response(format!("Failed to create channel manager: {}", e)),
    });

    let batcher = handle(FrameBatcher::new(ws_config.frame_rate));

    let mcp = handle(match McpServer::new(ws_config.mcp_enabled).await {
        Ok(m) => m,
        Err(e) => return error_response(format!("Failed to create MCP server: {}", e)),
    });

    // Create network server implementation
    let server_impl = handle(NetworkServer {
        websocket: websocket.clone(),
        channel_manager,
        batcher: batcher.clone(),
        mcp: mcp.clone(),
        config: shared(config.clone()),
        ws_config: ws_config.clone(),
        stats: shared(ServerStats::default()),
        running: shared(false),
        shutdown_signal: shared(None),
        start_time: std::time::Instant::now(),
    });

    // Store server implementation
    *NETWORK_STATE.server_impl.write().await = Some(server_impl.clone());

    // Update server entity's state component
    let world = match get_world().await {
        Ok(w) => w,
        Err(e) => return error_response(format!("Failed to get world: {}", e)),
    };

    // Get and modify the component through World
    if let Ok(mut state_comp) = server_entity.get_component::<ServerState>().await {
        let mut modified = state_comp;
        modified.is_running = true;
        let _ = server_entity.remove_component::<ServerState>().await;
        let _ = server_entity.add_component(modified).await;
    }

    // Start batch processing if enabled
    #[cfg(feature = "batching")]
    if config.enable_batching {
        let batcher_clone = batcher.clone();
        tokio::spawn(async move {
            batcher_clone.start_batch_loop().await;
        });
    }

    // Create Axum router
    let ws_routes = Router::new()
        .route("/ws", get(crate::websocket::websocket_handler))
        .with_state(websocket);

    let app = Router::new()
        .route("/", get(|| async { "Playground Server" }))
        .merge(ws_routes)
        .nest("/mcp", mcp.router());

    // Create shutdown channel
    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        *server_impl.shutdown_signal.write().await = Some(tx);
    }

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], ws_config.port));
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => return error_response(format!("Failed to bind to port: {}", e)),
    };

    // Mark as running
    {
        *server_impl.running.write().await = true;
    }

    // Spawn server task
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await;
    });

    success_response(None)
}

async fn handle_server_stop() -> VTableResponse {
    // Get server implementation
    let server_impl = match NETWORK_STATE.server_impl.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };

    // Send shutdown signal
    if let Some(tx) = server_impl.shutdown_signal.write().await.take() {
        let _ = tx.send(());
    }

    // Mark as not running
    *server_impl.running.write().await = false;

    // Update server entity's state
    if let Some(server_entity) = NETWORK_STATE.server_entity.read().await.as_ref() {
        if let Ok(state_comp) = server_entity.get_component::<ServerState>().await {
            let mut modified = state_comp;
            modified.is_running = false;
            let _ = server_entity.remove_component::<ServerState>().await;
            let _ = server_entity.add_component(modified).await;
        }

        // Stop the server entity
        let _ = server_api::stop_server(server_entity.downgrade()).await;
    }

    // Clear server state
    *NETWORK_STATE.server_impl.write().await = None;
    *NETWORK_STATE.server_entity.write().await = None;

    success_response(None)
}

async fn handle_send_to(payload: Bytes) -> VTableResponse {
    #[derive(serde::Deserialize)]
    struct SendToPayload {
        connection: ConnectionId,
        message: Message,
    }

    let params: SendToPayload = match bincode::deserialize(&payload) {
        Ok(p) => p,
        Err(e) => return error_response(format!("Failed to deserialize payload: {}", e)),
    };

    let server_impl = match NETWORK_STATE.server_impl.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };

    // Queue message in batcher
    #[cfg(feature = "batching")]
    server_impl.batcher.queue_message(params.connection, params.message).await;

    #[cfg(not(feature = "batching"))]
    {
        // Direct send without batching
        if let Some(sender) = NETWORK_STATE.connection_senders.read().await.get(&params.connection) {
            let data = bincode::serialize(&params.message).unwrap_or_default();
            let _ = sender.send(data).await;
        }
    }

    // Update stats in server entity
    if let Some(server_entity) = NETWORK_STATE.server_entity.read().await.as_ref() {
        if let Ok(stats_comp) = server_entity.get_component::<ServerStatsComponent>().await {
            let mut modified = stats_comp;
            modified.stats.total_messages_sent += 1;
            let _ = server_entity.remove_component::<ServerStatsComponent>().await;
            let _ = server_entity.add_component(modified).await;
        }
    }

    success_response(None)
}

async fn handle_broadcast(payload: Bytes) -> VTableResponse {
    let message: Message = match bincode::deserialize(&payload) {
        Ok(m) => m,
        Err(e) => return error_response(format!("Failed to deserialize message: {}", e)),
    };

    let server_impl = match NETWORK_STATE.server_impl.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };

    // Get all connections and send to each
    let connections = server_impl.websocket.get_all_connections().await;
    for conn in connections {
        #[cfg(feature = "batching")]
        server_impl.batcher.queue_message(conn.id, message.clone()).await;

        #[cfg(not(feature = "batching"))]
        {
            if let Some(sender) = NETWORK_STATE.connection_senders.read().await.get(&conn.id) {
                let data = bincode::serialize(&message).unwrap_or_default();
                let _ = sender.send(data).await;
            }
        }
    }

    success_response(None)
}

#[cfg(feature = "channels")]
async fn handle_publish(payload: Bytes) -> VTableResponse {
    #[derive(serde::Deserialize)]
    struct PublishPayload {
        channel: ChannelId,
        message: Message,
    }

    let params: PublishPayload = match bincode::deserialize(&payload) {
        Ok(p) => p,
        Err(e) => return error_response(format!("Failed to deserialize payload: {}", e)),
    };

    let server_impl = match NETWORK_STATE.server_impl.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };

    // Get subscribers and send to each
    let subscribers = server_impl.channel_manager.get_subscribers(params.channel.0).await;
    for conn_id in subscribers {
        #[cfg(feature = "batching")]
        server_impl.batcher.queue_message(conn_id, params.message.clone()).await;

        #[cfg(not(feature = "batching"))]
        {
            if let Some(sender) = NETWORK_STATE.connection_senders.read().await.get(&conn_id) {
                let data = bincode::serialize(&params.message).unwrap_or_default();
                let _ = sender.send(data).await;
            }
        }
    }

    success_response(None)
}

// Channel operation implementations

#[cfg(feature = "channels")]
async fn handle_subscribe(payload: Bytes) -> VTableResponse {
    #[derive(serde::Deserialize)]
    struct SubscribePayload {
        connection: ConnectionId,
        channel: ChannelId,
    }

    let params: SubscribePayload = match bincode::deserialize(&payload) {
        Ok(p) => p,
        Err(e) => return error_response(format!("Failed to deserialize payload: {}", e)),
    };

    let server_impl = match NETWORK_STATE.server_impl.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };

    match server_impl.channel_manager.subscribe(params.channel.0, params.connection).await {
        Ok(_) => success_response(None),
        Err(e) => error_response(format!("Failed to subscribe: {}", e)),
    }
}

#[cfg(feature = "channels")]
async fn handle_unsubscribe(payload: Bytes) -> VTableResponse {
    #[derive(serde::Deserialize)]
    struct UnsubscribePayload {
        connection: ConnectionId,
        channel: ChannelId,
    }

    let params: UnsubscribePayload = match bincode::deserialize(&payload) {
        Ok(p) => p,
        Err(e) => return error_response(format!("Failed to deserialize payload: {}", e)),
    };

    let server_impl = match NETWORK_STATE.server_impl.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };

    match server_impl.channel_manager.unsubscribe(params.channel.0, params.connection).await {
        Ok(_) => success_response(None),
        Err(e) => error_response(format!("Failed to unsubscribe: {}", e)),
    }
}

// Server event implementations

async fn handle_on_connection(payload: Bytes) -> VTableResponse {
    let address = match std::str::from_utf8(&payload) {
        Ok(a) => a.to_string(),
        Err(e) => return error_response(format!("Invalid address: {}", e)),
    };

    // Create connection entity
    let conn_entity = match server_api::accept_connection(address.clone()).await {
        Ok(entity) => entity,
        Err(e) => return error_response(format!("Failed to create connection entity: {}", e)),
    };

    // Get connection ID from the entity
    let conn_id = if let Ok(conn) = conn_entity.get_component::<ServerConnection>().await {
        conn.id
    } else {
        return error_response("Failed to get connection component".to_string());
    };

    // Store connection entity mapping
    NETWORK_STATE.connection_entities.write().await.insert(conn_id, conn_entity.clone());

    // Update server stats
    if let Some(server_entity) = NETWORK_STATE.server_entity.read().await.as_ref() {
        if let Ok(stats_comp) = server_entity.get_component::<ServerStatsComponent>().await {
            let mut modified = stats_comp;
            modified.stats.total_connections += 1;
            modified.stats.active_connections += 1;
            let _ = server_entity.remove_component::<ServerStatsComponent>().await;
            let _ = server_entity.add_component(modified).await;
        }
    }

    // Store in WebSocket handler
    if let Some(server_impl) = NETWORK_STATE.server_impl.read().await.as_ref() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let connection_info = ConnectionInfo {
            id: conn_id,
            established_at: now,
            last_activity: now,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            status: ConnectionStatus::Connected,
            metadata: std::collections::HashMap::new(),
        };
        server_impl.websocket.store_connection(connection_info).await;
    }

    success_response(None)
}

async fn handle_on_disconnection(payload: Bytes) -> VTableResponse {
    let id: ConnectionId = match bincode::deserialize(&payload) {
        Ok(i) => i,
        Err(e) => return error_response(format!("Failed to deserialize id: {}", e)),
    };

    // Remove connection entity
    if let Some(conn_entity) = NETWORK_STATE.connection_entities.write().await.remove(&id) {
        let _ = server_api::disconnect_connection(conn_entity.downgrade()).await;
    }

    // Remove connection sender
    NETWORK_STATE.connection_senders.write().await.remove(&id);

    // Update server stats
    if let Some(server_entity) = NETWORK_STATE.server_entity.read().await.as_ref() {
        if let Ok(stats_comp) = server_entity.get_component::<ServerStatsComponent>().await {
            let mut modified = stats_comp;
            modified.stats.active_connections = modified.stats.active_connections.saturating_sub(1);
            let _ = server_entity.remove_component::<ServerStatsComponent>().await;
            let _ = server_entity.add_component(modified).await;
        }
    }

    // Remove from WebSocket handler
    if let Some(server_impl) = NETWORK_STATE.server_impl.read().await.as_ref() {
        server_impl.websocket.remove_connection_by_core_id(id).await;
    }

    success_response(None)
}

// Client operation implementations

async fn handle_client_initialize(payload: Bytes) -> VTableResponse {
    let config: ClientConfig = match bincode::deserialize(&payload) {
        Ok(c) => c,
        Err(e) => return error_response(format!("Failed to deserialize config: {}", e)),
    };

    // Create client entity with components
    let client_entity = match client_api::initialize_client(config).await {
        Ok(entity) => entity,
        Err(e) => return error_response(format!("Failed to create client entity: {}", e)),
    };

    // Store client entity
    *NETWORK_STATE.client_entity.write().await = Some(client_entity.clone());

    success_response(None)
}

async fn handle_client_connect(payload: Bytes) -> VTableResponse {
    let address = match std::str::from_utf8(&payload) {
        Ok(a) => a,
        Err(e) => return error_response(format!("Invalid address: {}", e)),
    };

    // Get client entity
    let client_entity = match NETWORK_STATE.client_entity.read().await.as_ref() {
        Some(entity) => entity.clone(),
        None => return error_response("Client not initialized".to_string()),
    };

    // Parse WebSocket URL
    let url = match url::Url::parse(address) {
        Ok(u) => u,
        Err(e) => return error_response(format!("Invalid URL: {}", e)),
    };

    // Connect WebSocket client
    match tokio_tungstenite::connect_async(&url).await {
        Ok((_ws_stream, _)) => {
            // Update client state
            if let Ok(state_comp) = client_entity.get_component::<ClientStateComponent>().await {
                let mut modified = state_comp;
                modified.state = ClientState::Connected;
                modified.server_address = Some(address.to_string());
                modified.connected_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                );
                let _ = client_entity.remove_component::<ClientStateComponent>().await;
                let _ = client_entity.add_component(modified).await;
            }

            // TODO: Store WebSocket stream for client use
            success_response(None)
        }
        Err(e) => error_response(format!("Failed to connect: {}", e)),
    }
}

async fn handle_client_disconnect() -> VTableResponse {
    // Get client entity
    let client_entity = match NETWORK_STATE.client_entity.read().await.as_ref() {
        Some(entity) => entity.clone(),
        None => return error_response("Client not initialized".to_string()),
    };

    // Update client state
    if let Ok(state_comp) = client_entity.get_component::<ClientStateComponent>().await {
        let mut modified = state_comp;
        modified.state = ClientState::Disconnected;
        modified.server_address = None;
        modified.connected_at = None;
        let _ = client_entity.remove_component::<ClientStateComponent>().await;
        let _ = client_entity.add_component(modified).await;
    }

    // TODO: Close WebSocket connection
    success_response(None)
}

async fn handle_client_send(_payload: Bytes) -> VTableResponse {
    // Get client entity
    let client_entity = match NETWORK_STATE.client_entity.read().await.as_ref() {
        Some(entity) => entity.clone(),
        None => return error_response("Client not initialized".to_string()),
    };

    // Check if connected
    if let Ok(state) = client_entity.get_component::<ClientStateComponent>().await {
        if state.state != ClientState::Connected {
            return error_response("Client not connected".to_string());
        }
    }

    // TODO: Send data through WebSocket
    success_response(None)
}

async fn handle_client_receive() -> VTableResponse {
    // Get client entity
    let client_entity = match NETWORK_STATE.client_entity.read().await.as_ref() {
        Some(entity) => entity.clone(),
        None => return error_response("Client not initialized".to_string()),
    };

    // Check if connected
    if let Ok(state) = client_entity.get_component::<ClientStateComponent>().await {
        if state.state != ClientState::Connected {
            return error_response("Client not connected".to_string());
        }
    }

    // TODO: Receive data from WebSocket
    success_response(Some(Bytes::new()))
}

async fn handle_client_update(payload: Bytes) -> VTableResponse {
    let _delta_time: f32 = match bincode::deserialize(&payload) {
        Ok(dt) => dt,
        Err(e) => return error_response(format!("Failed to deserialize delta_time: {}", e)),
    };

    // Get client entity
    let client_entity = match NETWORK_STATE.client_entity.read().await.as_ref() {
        Some(entity) => entity.clone(),
        None => return error_response("Client not initialized".to_string()),
    };

    // Update client stats
    if let Ok(stats_comp) = client_entity.get_component::<ClientStatsComponent>().await {
        let mut modified = stats_comp;
        modified.stats.total_frames += 1;
        let _ = client_entity.remove_component::<ClientStatsComponent>().await;
        let _ = client_entity.add_component(modified).await;
    }

    success_response(None)
}