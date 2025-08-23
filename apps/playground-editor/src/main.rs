// mod messages;
// mod message_bus;
// mod layout;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use playground_systems_logic::{World, SystemsManager, System};

#[tokio::main]
async fn main() -> Result<()> {
    // Create the World from systems/logic
    let world = Arc::new(RwLock::new(World::new()));
    
    // Create SystemsManager
    let systems = Arc::new(SystemsManager::new(world.clone()).await?);
    eprintln!("[MAIN] SystemsManager created");
    systems.initialize_all().await?;
    eprintln!("[MAIN] SystemsManager.initialize_all() completed");
    
    // Check if UI system has root element
    {
        let ui = systems.ui();
        let ui_read = ui.read().await;
        let has_root = ui_read.get_root_element().is_some();
        eprintln!("[MAIN] UI System has root element: {}", has_root);
        eprintln!("[MAIN] UI System initialized: {}", ui_read.is_initialized());
    }
    
    // Load and register the UI Framework Plugin as a System
    use playground_plugins_ui_framework::UiFrameworkPlugin;
    
    eprintln!("[MAIN] Creating UI Framework Plugin...");
    let mut ui_plugin = UiFrameworkPlugin::new(systems.clone());
    eprintln!("[MAIN] UI Framework Plugin created");
    
    // Initialize the plugin
    eprintln!("[MAIN] Calling ui_plugin.initialize()...");
    match ui_plugin.initialize(&*world.read().await).await {
        Ok(_) => eprintln!("[MAIN] ✓ UI Framework Plugin initialized successfully"),
        Err(e) => {
            eprintln!("[MAIN] ✗ Failed to initialize UI Framework Plugin: {}", e);
            return Err(e.into());
        }
    }
    eprintln!("[MAIN] After ui_plugin.initialize()");
    
    // Register the plugin as a System in the World
    world.write().await.register_plugin_system(Box::new(ui_plugin)).await?;
    
    // Note: render loop already started in systems.initialize_all()
    
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