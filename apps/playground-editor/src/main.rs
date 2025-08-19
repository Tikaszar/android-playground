// mod messages;
// mod message_bus;
// mod layout;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use playground_systems_logic::{World, SystemsManager};

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
    
    // Create the World from systems/logic
    info!("Creating World from systems/logic...");
    let world = Arc::new(RwLock::new(World::new()));
    info!("✓ World created");
    
    // Create SystemsManager which initializes ALL engine systems
    info!("Initializing all engine systems...");
    let systems = Arc::new(SystemsManager::new(world.clone()).await?);
    systems.initialize_all().await?;
    info!("✓ All systems initialized:");
    info!("  - NetworkingSystem (starts core/server internally)");
    info!("  - UiSystem (using core/ecs internally)");
    info!("  - RenderingSystem (skipped - browser-side only)");
    
    // Load and register the UI Framework Plugin as a System
    info!("Loading UI Framework Plugin...");
    use playground_plugins_ui_framework::UiFrameworkPlugin;
    
    let ui_plugin = Box::new(UiFrameworkPlugin::new(systems.clone()));
    
    // Register the plugin as a System in the World
    world.write().await.register_plugin_system(ui_plugin).await?;
    info!("✓ UI Framework Plugin registered as System");
    info!("✓ Plugin will register channels 1200-1209 during initialization");
    
    // Start the main update loop that runs all Systems
    let world_for_update = world.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16)); // ~60fps
        loop {
            interval.tick().await;
            
            // Run all registered Systems
            // Use a block to ensure the lock is dropped before the next iteration
            {
                let mut world_lock = world_for_update.write().await;
                let _ = world_lock.run_systems(0.016).await;
            }
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
    info!("      ↓ creates systems/logic World");
    info!("      ↓ creates SystemsManager");
    info!("      ↓ SystemsManager initializes all systems");
    info!("      ↓ NetworkingSystem starts core/server internally");
    info!("      ↓ Plugins register as Systems in World");
    info!("");
    info!("Everything is running in this single process.");
    info!("=========================================");
    
    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Conversational IDE...");
    
    Ok(())
}