use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use playground_systems_logic::{World, SystemsManager, System, SystemData};

// Import all IDE plugins - each is self-contained
use playground_plugins_ui_framework::UiFrameworkPlugin;
use playground_plugins_editor_core::EditorCorePlugin;
use playground_plugins_file_browser::FileBrowserPlugin;
use playground_plugins_terminal::TerminalPlugin;
use playground_plugins_lsp_client::LspClientPlugin;
use playground_plugins_debugger::DebuggerPlugin;
use playground_plugins_chat_assistant::ChatAssistantPlugin;
use playground_plugins_version_control::VersionControlPlugin;
use playground_plugins_theme_manager::ThemeManagerPlugin;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with simple setup
    tracing_subscriber::fmt::init();

    // Create the World from systems/logic
    let world = Arc::new(RwLock::new(World::new()));
    
    // Create SystemsManager which initializes all engine systems
    let systems = Arc::new(SystemsManager::new(world.clone()).await?);
    eprintln!("[EDITOR] SystemsManager created");
    systems.initialize_all().await?;
    eprintln!("[EDITOR] All engine systems initialized");
    
    // Verify UI system is ready
    {
        let ui = systems.ui();
        let ui_read = ui.read().await;
        eprintln!("[EDITOR] UI System ready: {}", ui_read.is_initialized());
    }
    
    // Load and register all IDE plugins as Systems
    // The App coordinates all plugins - they don't depend on each other
    
    eprintln!("[EDITOR] Loading IDE plugins...");
    
    // 1. UI Framework - Discord-style mobile UI (channels 1200-1209)
    {
        let mut plugin = UiFrameworkPlugin::new(systems.clone());
        plugin.initialize(&*world.read().await).await?;
        world.write().await.register_plugin_system(SystemData::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ UI Framework Plugin loaded (Discord UI)");
    }
    
    // 2. Editor Core - Text editing with vim mode (channel 1000)
    {
        let mut plugin = EditorCorePlugin::new(systems.clone());
        plugin.initialize(&*world.read().await).await?;
        world.write().await.register_plugin_system(SystemData::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ Editor Core Plugin loaded");
    }
    
    // 3. File Browser - File navigation (channel 1001)
    {
        let mut plugin = FileBrowserPlugin::new(systems.clone());
        plugin.initialize(&*world.read().await).await?;
        world.write().await.register_plugin_system(SystemData::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ File Browser Plugin loaded");
    }
    
    // 4. Terminal - Termux integration (channel 1002)
    {
        let mut plugin = TerminalPlugin::new(systems.clone());
        plugin.initialize(&*world.read().await).await?;
        world.write().await.register_plugin_system(SystemData::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ Terminal Plugin loaded");
    }
    
    // 5. LSP Client - Language server protocol (channel 1003)
    {
        let mut plugin = LspClientPlugin::new(systems.clone());
        plugin.initialize(&*world.read().await).await?;
        world.write().await.register_plugin_system(SystemData::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ LSP Client Plugin loaded");
    }
    
    // 6. Debugger - Debug support (channel 1004)
    {
        let mut plugin = DebuggerPlugin::new(systems.clone());
        plugin.initialize(&*world.read().await).await?;
        world.write().await.register_plugin_system(SystemData::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ Debugger Plugin loaded");
    }
    
    // 7. Chat Assistant - MCP/LLM integration (channel 1005)
    {
        let mut plugin = ChatAssistantPlugin::new(systems.clone());
        plugin.initialize(&*world.read().await).await?;
        world.write().await.register_plugin_system(SystemData::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ Chat Assistant Plugin loaded (MCP integration)");
    }
    
    // 8. Version Control - Git integration (channel 1006)
    {
        let mut plugin = VersionControlPlugin::new(systems.clone());
        plugin.initialize(&*world.read().await).await?;
        world.write().await.register_plugin_system(SystemData::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ Version Control Plugin loaded");
    }
    
    // 9. Theme Manager - UI theming (channel 1007)
    {
        let mut plugin = ThemeManagerPlugin::new(systems.clone());
        plugin.initialize(&*world.read().await).await?;
        world.write().await.register_plugin_system(SystemData::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ Theme Manager Plugin loaded");
    }
    
    eprintln!("[EDITOR] All IDE plugins loaded successfully!");
    eprintln!("[EDITOR] Channel allocations:");
    eprintln!("  - 1000: Editor Core");
    eprintln!("  - 1001: File Browser");
    eprintln!("  - 1002: Terminal");
    eprintln!("  - 1003: LSP Client");
    eprintln!("  - 1004: Debugger");
    eprintln!("  - 1005: Chat Assistant (MCP)");
    eprintln!("  - 1006: Version Control");
    eprintln!("  - 1007: Theme Manager");
    eprintln!("  - 1200-1209: UI Framework");
    
    // Start the main update loop that runs all Systems (including plugins)
    let world_for_update = world.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16)); // ~60fps
        loop {
            interval.tick().await;
            
            // Run all registered Systems
            {
                let mut world_lock = world_for_update.write().await;
                if let Err(e) = world_lock.run_systems(0.016).await {
                    eprintln!("[EDITOR] Error running systems: {}", e);
                }
            }
        }
    });
    
    eprintln!("[EDITOR] Main update loop started");
    eprintln!("[EDITOR] IDE interface served at: http://localhost:8080/playground-editor/");
    eprintln!("[EDITOR] MCP endpoint: http://localhost:8080/mcp");
    eprintln!("[EDITOR] Press Ctrl+C to shutdown");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    eprintln!("[EDITOR] Shutting down...");
    
    // Cleanup all systems
    {
        let mut world_lock = world.write().await;
        // World doesn't have a shutdown method - cleanup handled by drop
    }
    
    eprintln!("[EDITOR] Shutdown complete");
    Ok(())
}