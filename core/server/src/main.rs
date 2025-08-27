use axum::{
    routing::{get, post},
    Router,
    http::{header, HeaderValue},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing_subscriber;

use playground_core_server::{
    list_plugins, reload_plugin, root,
    websocket_handler, WebSocketState,
    McpServer,
};

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
        .nest_service("/playground-editor", 
            ServeDir::new("apps/playground-editor/static")
                .append_index_html_on_directories(true)
        )
        .nest("/mcp", mcp_router)  // Mount MCP endpoints under /mcp
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        // Force no caching - always get fresh files
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::PRAGMA,
            HeaderValue::from_static("no-cache"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::EXPIRES,
            HeaderValue::from_static("0"),
        ))
        .with_state(ws_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Server listening on {}", addr);
    tracing::info!("WebSocket endpoint: ws://localhost:8080/ws");
    tracing::info!("MCP endpoint: http://localhost:8080/mcp");
    tracing::info!("Playground Editor: http://localhost:8080/playground-editor/");
    tracing::info!("Connect LLMs with: --mcp http://localhost:8080/mcp");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}