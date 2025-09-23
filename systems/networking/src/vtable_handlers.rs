//! VTable handlers for server and client operations
//!
//! This module implements all the actual networking logic that core/server
//! and core/client delegate to via VTable.

use std::collections::HashMap;
use bytes::Bytes;
use playground_core_types::{Handle, handle, Shared, shared, CoreResult};
use playground_core_ecs::VTableResponse;
use playground_core_server::{
    ServerConfig, ServerStats, ConnectionId, ConnectionInfo, ConnectionStatus,
    Message, MessageId, MessagePriority, ChannelId, ChannelInfo,
};
use playground_core_client::{ClientConfig, ClientState, ClientStats};
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use std::net::SocketAddr;
use once_cell::sync::Lazy;

use crate::server::NetworkServer;
use crate::types::WebSocketConfig;
use crate::websocket::WebSocketHandler;
use crate::channel_manager::ChannelManager;
use crate::batcher::FrameBatcher;
use crate::mcp::McpServer;

// Network state using Lazy initialization
static NETWORK_STATE: Lazy<NetworkState> = Lazy::new(|| NetworkState {
    server: shared(None),
    client_connections: shared(HashMap::new()),
});

struct NetworkState {
    server: Shared<Option<Handle<NetworkServer>>>,
    client_connections: Shared<HashMap<ConnectionId, tokio::sync::mpsc::Sender<Vec<u8>>>>,
}

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
    // Force lazy initialization by accessing NETWORK_STATE
    let _ = &*NETWORK_STATE;
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

// Note: Rendering operations removed - they belong in systems/webgl

// Note: Input operations removed - they belong in systems/input

// Note: Audio operations removed - they belong in systems/audio

// Server operation implementations

async fn handle_server_start(payload: Bytes) -> VTableResponse {
    let config: ServerConfig = match bincode::deserialize(&payload) {
        Ok(c) => c,
        Err(e) => return error_response(format!("Failed to deserialize config: {}", e)),
    };
    
    // Create WebSocket configuration with defaults
    let ws_config = WebSocketConfig {
        port: 8080, // Default port, since ServerConfig doesn't have one
        frame_rate: 60,
        max_connections: config.max_connections,
        max_message_size: config.max_message_size,
        mcp_enabled: true,
    };
    
    // Create server components
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
    
    // Create server instance
    let server = handle(NetworkServer {
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
    
    // Store server instance
    *NETWORK_STATE.server.write().await = Some(server.clone());

    // Update core/server state
    match playground_core_server::get_server_instance() {
        Ok(core_server) => {
            *core_server.is_running.write().await = true;
        },
        Err(e) => return error_response(format!("Failed to get server instance: {}", e)),
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
        *server.shutdown_signal.write().await = Some(tx);
    }
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], ws_config.port));
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => return error_response(format!("Failed to bind to port: {}", e)),
    };
    
    // Mark as running
    {
        *server.running.write().await = true;
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
    let server = match NETWORK_STATE.server.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };

    // Send shutdown signal
    if let Some(tx) = server.shutdown_signal.write().await.take() {
        let _ = tx.send(());
    }

    // Mark as not running
    *server.running.write().await = false;

    // Clear server instance
    *NETWORK_STATE.server.write().await = None;

    // Update core/server state
    if let Ok(core_server) = playground_core_server::get_server_instance() {
        *core_server.is_running.write().await = false;
    }

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
    
    let server = match NETWORK_STATE.server.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };
    
    // Queue message in batcher
    #[cfg(feature = "batching")]
    server.batcher.queue_message(params.connection, params.message).await;
    
    #[cfg(not(feature = "batching"))]
    {
        // Direct send without batching
        // Implementation would go here
    }
    
    // Update stats
    {
        let mut stats = server.stats.write().await;
        stats.total_messages_sent += 1;
    }
    
    success_response(None)
}

async fn handle_broadcast(payload: Bytes) -> VTableResponse {
    let message: Message = match bincode::deserialize(&payload) {
        Ok(m) => m,
        Err(e) => return error_response(format!("Failed to deserialize message: {}", e)),
    };
    
    let server = match NETWORK_STATE.server.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };
    
    // Get all connections and send to each
    let connections = server.websocket.get_all_connections().await;
    for conn in connections {
        #[cfg(feature = "batching")]
        server.batcher.queue_message(conn.id, message.clone()).await;
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
    
    let server = match NETWORK_STATE.server.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };
    
    // Get subscribers and send to each
    let subscribers = server.channel_manager.get_subscribers(params.channel.0).await;
    for conn_id in subscribers {
        #[cfg(feature = "batching")]
        server.batcher.queue_message(conn_id, params.message.clone()).await;
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
    
    let server = match NETWORK_STATE.server.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };
    
    match server.channel_manager.subscribe(params.channel.0, params.connection).await {
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
    
    let server = match NETWORK_STATE.server.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };
    
    match server.channel_manager.unsubscribe(params.channel.0, params.connection).await {
        Ok(_) => success_response(None),
        Err(e) => error_response(format!("Failed to unsubscribe: {}", e)),
    }
}

// Server event implementations

async fn handle_on_connection(payload: Bytes) -> VTableResponse {
    let connection: ConnectionInfo = match bincode::deserialize(&payload) {
        Ok(c) => c,
        Err(e) => return error_response(format!("Failed to deserialize connection: {}", e)),
    };
    
    let server = match NETWORK_STATE.server.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };
    
    // Update stats
    {
        let mut stats = server.stats.write().await;
        stats.total_connections += 1;
        stats.active_connections += 1;
    }
    
    // Store connection
    server.websocket.store_connection(connection).await;
    
    success_response(None)
}

async fn handle_on_disconnection(payload: Bytes) -> VTableResponse {
    let id: ConnectionId = match bincode::deserialize(&payload) {
        Ok(i) => i,
        Err(e) => return error_response(format!("Failed to deserialize id: {}", e)),
    };
    
    let server = match NETWORK_STATE.server.read().await.as_ref() {
        Some(s) => s.clone(),
        None => return error_response("Server not running".to_string()),
    };
    
    // Update stats
    {
        let mut stats = server.stats.write().await;
        stats.active_connections = stats.active_connections.saturating_sub(1);
    }
    
    // Remove connection
    server.websocket.remove_connection_by_core_id(id).await;
    
    success_response(None)
}

// Client operation implementations

async fn handle_client_initialize(payload: Bytes) -> VTableResponse {
    let _config: ClientConfig = match bincode::deserialize(&payload) {
        Ok(c) => c,
        Err(e) => return error_response(format!("Failed to deserialize config: {}", e)),
    };
    
    // Client connections are already initialized via Lazy
    // Just access to ensure initialization
    let _ = &*NETWORK_STATE;
    
    success_response(None)
}

async fn handle_client_connect(payload: Bytes) -> VTableResponse {
    let address = match std::str::from_utf8(&payload) {
        Ok(a) => a,
        Err(e) => return error_response(format!("Invalid address: {}", e)),
    };
    
    // Parse WebSocket URL
    let url = match url::Url::parse(address) {
        Ok(u) => u,
        Err(e) => return error_response(format!("Invalid URL: {}", e)),
    };
    
    // Connect WebSocket client
    match tokio_tungstenite::connect_async(&url).await {
        Ok((_ws_stream, _)) => {
            // Store connection for client use
            // Implementation would handle the WebSocket stream
            success_response(None)
        }
        Err(e) => error_response(format!("Failed to connect: {}", e)),
    }
}

async fn handle_client_disconnect() -> VTableResponse {
    // Close WebSocket connection
    // Implementation would close the active connection
    success_response(None)
}

async fn handle_client_send(_payload: Bytes) -> VTableResponse {
    // Send data through WebSocket
    // Implementation would send through active connection
    success_response(None)
}

async fn handle_client_receive() -> VTableResponse {
    // Receive data from WebSocket
    // Implementation would receive from active connection
    success_response(Some(Bytes::new()))
}

async fn handle_client_update(payload: Bytes) -> VTableResponse {
    let _delta_time: f32 = match bincode::deserialize(&payload) {
        Ok(dt) => dt,
        Err(e) => return error_response(format!("Failed to deserialize delta_time: {}", e)),
    };
    
    // Update client state
    // Implementation would update internal state
    success_response(None)
}

// Input operation implementations removed - they belong in systems/input
