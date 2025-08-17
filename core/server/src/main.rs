mod handlers;
mod channel;
mod packet;
mod batcher;
mod websocket;
mod mcp;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber;

use handlers::{list_plugins, reload_plugin, root};
use websocket::{websocket_handler, WebSocketState};
use mcp::McpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let ws_state = Arc::new(WebSocketState::new());

    // Create MCP server
    let mcp_server = McpServer::new();
    let mcp_router = mcp_server.router();

    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(websocket_handler))
        .route("/api/plugins", get(list_plugins))
        .route("/api/reload", post(reload_plugin))
        .nest_service("/test", ServeDir::new("."))
        .nest("/mcp", mcp_router)  // Mount MCP endpoints under /mcp
        .layer(CorsLayer::permissive())
        .with_state(ws_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Server listening on {}", addr);
    tracing::info!("WebSocket endpoint: ws://localhost:8080/ws");
    tracing::info!("MCP endpoint: http://localhost:8080/mcp");
    tracing::info!("Connect LLMs with: --mcp http://localhost:8080/mcp");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}