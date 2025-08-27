use anyhow::Result;

use playground_systems_logic::{World, SystemsManager, handle, shared};

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
    let world = shared(World::new());
    
    // Create SystemsManager which initializes all engine systems
    let systems = handle(SystemsManager::new(world.clone()).await?);
    eprintln!("[EDITOR] SystemsManager created");

    // Verify UI system is ready
    {
        let ui = systems.ui();
        let ui_read = ui.read().await;
        eprintln!("[EDITOR] UI System ready: {}", ui_read.is_initialized());
    }

    // Phase 1: REGISTRATION - Register all IDE plugins WITHOUT initializing them
    // This ensures all plugins are known before any initialization happens
    eprintln!("[EDITOR] Phase 1: Registering IDE plugins...");

    // 1. UI Framework - Discord-style mobile UI
    {
        let plugin = UiFrameworkPlugin::new(systems.clone());
        world.write().await.register_plugin_system(Box::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ UiFrameworkPlugin registered (not initialized)");
    }

    // 2. Editor Core - Text editing with vim mode
    {
        let plugin = EditorCorePlugin::new(systems.clone());
        world.write().await.register_plugin_system(Box::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ EditorCorePlugin registered (not initialized)");
    }

    // 3. File Browser - File navigation
    {
        let plugin = FileBrowserPlugin::new(systems.clone());
        world.write().await.register_plugin_system(Box::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ FileBrowserPlugin registered (not initialized)");
    }

    // 4. Terminal - Termux integration
    {
        let plugin = TerminalPlugin::new(systems.clone());
        world.write().await.register_plugin_system(Box::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ TerminalPlugin registered (not initialized)");
    }

    // 5. LSP Client - Language server protocol
    {
        let plugin = LspClientPlugin::new(systems.clone());
        world.write().await.register_plugin_system(Box::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ LspClientPlugin registered (not initialized)");
    }

    // 6. Debugger - Debug support
    {
        let plugin = DebuggerPlugin::new(systems.clone());
        world.write().await.register_plugin_system(Box::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ DebuggerPlugin registered (not initialized)");
    }

    // 7. Chat Assistant - MCP/LLM integration
    {
        let plugin = ChatAssistantPlugin::new(systems.clone());
        world.write().await.register_plugin_system(Box::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ ChatAssistantPlugin registered (not initialized)");
    }

    // 8. Version Control - Git integration
    {
        let plugin = VersionControlPlugin::new(systems.clone());
        world.write().await.register_plugin_system(Box::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ VersionControlPlugin registered (not initialized)");
    }

    // 9. Theme Manager - UI theming
    {
        let plugin = ThemeManagerPlugin::new(systems.clone());
        world.write().await.register_plugin_system(Box::new(plugin)).await?;
        eprintln!("[EDITOR] ✓ ThemeManagerPlugin registered (not initialized)");
    }

    eprintln!("[EDITOR] Phase 1 complete: All IDE plugins registered!");

    // Phase 2: CORE INITIALIZATION - Initialize core engine systems
    // Now that all plugins are registered, the NetworkingSystem can build
    // a complete channel manifest
    eprintln!("[EDITOR] Phase 2: Initializing core engine systems...");
    systems.initialize_all().await?;
    eprintln!("[EDITOR] Phase 2 complete: Core engine systems initialized!");

    // Phase 3: PLUGIN INITIALIZATION - Initialize all registered plugins
    // Now that NetworkingSystem is ready, plugins can perform network operations
    eprintln!("[EDITOR] Phase 3: Initializing all plugins...");
    world.write().await.initialize_all_plugins().await?;
    eprintln!("[EDITOR] Phase 3 complete: All plugins initialized!");
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
        world_lock.shutdown().await?;
    }
    
    eprintln!("[EDITOR] Shutdown complete");
    Ok(())
}