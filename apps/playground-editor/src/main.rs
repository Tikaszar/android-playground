// mod messages;
// mod message_bus;
// mod layout;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use playground_systems_logic::{World, SystemsManager};

#[tokio::main]
async fn main() -> Result<()> {
    // Create the World from systems/logic
    let world = Arc::new(RwLock::new(World::new()));
    
    // Create SystemsManager
    let systems = Arc::new(SystemsManager::new(world.clone()).await?);
    systems.initialize_all().await?;
    
    // Load and register the UI Framework Plugin as a System
    use playground_plugins_ui_framework::UiFrameworkPlugin;
    
    let ui_plugin = Box::new(UiFrameworkPlugin::new(systems.clone()));
    
    // Register the plugin as a System in the World
    world.write().await.register_plugin_system(ui_plugin).await?;
    
    // Start the render loop for UI system
    systems.start_render_loop().await?;
    
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
    
    // Dashboard will show all status - just wait 100ms for it to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Keep the application running
    tokio::signal::ctrl_c().await?;
    // Dashboard will handle shutdown message
    
    Ok(())
}