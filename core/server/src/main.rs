mod handlers;
mod channel;
mod packet;
mod batcher;
mod websocket;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let ws_state = Arc::new(WebSocketState::new());

    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(websocket_handler))
        .route("/api/plugins", get(list_plugins))
        .route("/api/reload", post(reload_plugin))
        .nest_service("/test", ServeDir::new("."))
        .layer(CorsLayer::permissive())
        .with_state(ws_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server listening on {}", addr);
    tracing::info!("WebSocket endpoint: ws://localhost:3000/ws");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}