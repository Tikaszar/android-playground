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
    info!("This is the Conversational IDE for Android Playground.");
    info!("It provides a Discord-style chat interface for development.");
    info!("");
    
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
    
    // Note: In a complete implementation:
    // 1. Load the UI Framework Plugin using the Plugin trait
    // 2. Pass the systems to the plugin through its Context
    // 3. The plugin uses the provided systems (NEVER creates its own)
    // 4. The plugin uses systems/networking to handle WebSocket messages
    
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
    info!("Conversational IDE Architecture:");
    info!("");
    info!("  playground-editor (App)");
    info!("      ↓ creates");
    info!("  systems/logic (ECS)");
    info!("      ↓ creates core/ecs internally");
    info!("      ↓ initializes ALL systems");
    info!("  systems/networking");
    info!("      ↓ creates and manages");
    info!("  core/server connection");
    info!("");
    info!("Web Interface: http://localhost:3001");
    info!("WebSocket: ws://localhost:8080/ws");
    info!("Channels: 1200-1209 (UI Framework)");
    info!("=========================================");
    
    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Conversational IDE...");
    
    Ok(())
}