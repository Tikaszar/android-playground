// mod messages;
// mod message_bus;
// mod layout;

use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tokio::sync::RwLock;
use tracing::info;

use playground_logic::ECS;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .init();
    
    info!("===========================================");
    info!("  Playground Editor - Conversational IDE  ");
    info!("===========================================");
    info!("");
    
    // Start the core server internally
    info!("Starting core server on port 8080...");
    tokio::spawn(async {
        // Run the core server
        if let Err(e) = run_core_server().await {
            tracing::error!("Core server failed: {}", e);
        }
    });
    
    // Give the core server time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Initialize the ECS system from systems/logic
    info!("Creating ECS from systems/logic...");
    let mut ecs = ECS::new();
    info!("✓ ECS created (with core/ecs internally)");
    
    // Initialize ALL systems through systems/logic
    info!("Initializing all engine systems...");
    let systems = ecs.initialize_systems().await?;
    info!("✓ All systems initialized:");
    info!("  - NetworkingSystem (connected to core/server)");
    info!("  - UiSystem (using core/ecs internally)");
    info!("  - RenderingSystem (using core/ecs internally)");
    
    // Register UI Framework Plugin channels
    info!("Registering UI Framework Plugin channels...");
    systems.register_plugin_channels("ui-framework", 1200, 10).await?;
    info!("✓ Registered channels 1200-1209 for UI Framework Plugin");
    
    // Store the ECS and systems for plugin usage
    let ecs = Arc::new(RwLock::new(ecs));
    let systems = systems.clone();
    
    // Start the web server for the IDE interface
    tokio::spawn(async move {
        // Serve static files for the Conversational IDE interface
        let static_dir = "apps/playground-editor/static";
        info!("Serving Conversational IDE interface from: {}", static_dir);
        
        let app = Router::new()
            .nest_service("/", ServeDir::new(static_dir))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());
        
        let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
        info!("Web interface starting on: http://localhost:3001");
        
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
    
    info!("=========================================");
    info!("Conversational IDE is running!");
    info!("");
    info!("Open your browser to: http://localhost:3001");
    info!("");
    info!("Architecture:");
    info!("  playground-editor (App)");
    info!("      ↓ starts core/server internally");
    info!("      ↓ creates systems/logic");
    info!("      ↓ systems/logic initializes all systems");
    info!("      ↓ systems/networking connects to core/server");
    info!("");
    info!("Everything is running in this single process.");
    info!("=========================================");
    
    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Conversational IDE...");
    
    Ok(())
}

// Run the core server internally
async fn run_core_server() -> Result<()> {
    use playground_server::WebSocketState;
    use playground_server::mcp::McpServer;
    use playground_server::websocket::websocket_handler;
    use playground_server::handlers::{list_plugins, reload_plugin, root};
    use axum::routing::{get, post};
    
    let ws_state = Arc::new(WebSocketState::new());
    
    // Create MCP server
    let mcp_server = McpServer::new();
    let mcp_router = mcp_server.router();
    
    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(websocket_handler))
        .route("/api/plugins", get(list_plugins))
        .route("/api/reload", post(reload_plugin))
        .nest("/mcp", mcp_router)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(ws_state);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Core server listening on {}", addr);
    tracing::info!("WebSocket endpoint: ws://localhost:8080/ws");
    tracing::info!("MCP endpoint: http://localhost:8080/mcp");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}