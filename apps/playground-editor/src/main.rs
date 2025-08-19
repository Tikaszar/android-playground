// mod messages;
// mod message_bus;
// mod layout;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use playground_systems_logic::ECS;

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
    info!("  - RenderingSystem (skipped - browser-side only)");
    
    // Register UI Framework Plugin channels
    info!("Registering UI Framework Plugin channels...");
    systems.register_plugin_channels("ui-framework", 1200, 10).await?;
    info!("✓ Registered channels 1200-1209 for UI Framework Plugin");
    
    // Store the ECS and systems for plugin usage
    let ecs = Arc::new(RwLock::new(ecs));
    let systems_clone = systems.clone();
    
    // Load and start the UI Framework Plugin
    info!("Loading UI Framework Plugin...");
    use playground_plugins_ui_framework::UiFrameworkPlugin;
    use playground_core_plugin::Plugin;
    use playground_core_types::context::Context;
    use playground_core_types::render_context::RenderContext;
    
    let mut ui_plugin = UiFrameworkPlugin::new();
    
    // Create plugin context with access to systems
    let mut context = Context::new();
    
    // Add NetworkingSystem to the context for the plugin to use
    // This follows the architecture: Apps pass Systems to Plugins
    context.resources.insert(
        "networking".to_string(),
        Box::new(systems.networking.clone())
    );
    context.resources.insert(
        "ui".to_string(),
        Box::new(systems.ui.clone())
    );
    // Rendering is browser-side only (WebGL isn't thread-safe)
    
    // Initialize the plugin with access to systems
    ui_plugin.on_load(&mut context).await?;
    info!("✓ UI Framework Plugin loaded and ready");
    
    // Start plugin update loop
    let ui_plugin = Arc::new(RwLock::new(ui_plugin));
    let ui_plugin_clone = ui_plugin.clone();
    let systems_for_update = systems.clone();
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16)); // ~60fps
        loop {
            interval.tick().await;
            let mut plugin = ui_plugin_clone.write().await;
            
            // Create context with systems for update
            let mut ctx = Context::new();
            ctx.resources.insert(
                "networking".to_string(),
                Box::new(systems_for_update.networking.clone())
            );
            
            plugin.update(&mut ctx, 0.016).await;
        }
    });
    
    // Note: The IDE interface is served by core/server at /playground-editor/
    
    info!("=========================================");
    info!("Conversational IDE is running!");
    info!("");
    info!("Open your browser to: http://localhost:8080/playground-editor/");
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
    use playground_core_server::{
        McpServer, WebSocketState, websocket_handler,
        list_plugins, reload_plugin, root
    };
    use axum::{Router, routing::{get, post}};
    use std::net::SocketAddr;
    use tower_http::cors::CorsLayer;
    use tower_http::services::ServeDir;
    use tower_http::trace::TraceLayer;
    
    let ws_state = Arc::new(WebSocketState::new());
    
    // Create MCP server
    let mcp_server = McpServer::new();
    let mcp_router = mcp_server.router();
    
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
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Core server listening on {}", addr);
    tracing::info!("WebSocket endpoint: ws://localhost:8080/ws");
    tracing::info!("MCP endpoint: http://localhost:8080/mcp");
    tracing::info!("Playground Editor: http://localhost:8080/playground-editor/");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}