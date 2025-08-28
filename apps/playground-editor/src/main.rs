use anyhow::Result;

use playground_systems_logic::{World, SystemsManager, System, handle, shared};
use playground_core_types::Handle;

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
    systems.log_component("apps/playground-editor", playground_systems_logic::LogLevel::Info,
        "SystemsManager created".to_string()).await;

    // Verify UI system is ready
    {
        let ui = systems.ui();
        let ui_read = ui.read().await;
        systems.log_component("apps/playground-editor", playground_systems_logic::LogLevel::Info,
            format!("UI System ready: {}", ui_read.is_initialized())).await;
    }

    // Phase 1: REGISTRATION - Register all IDE plugins and get their channels
    // This ensures all plugins are known before any initialization happens
    systems.log_component("apps/playground-editor", playground_systems_logic::LogLevel::Info,
        "Phase 1: Registering IDE plugins...".to_string()).await;

    // CRITICAL: Register plugin channels with SystemsManager's ChannelRegistry
    // This must happen BEFORE core initialization so the channel manifest is complete
    
    // 1. UI Framework - Discord-style mobile UI
    let ui_framework_channel = systems.register_plugin("ui-framework").await?;
    world.write().await.register_plugin_channel("ui-framework".to_string(), ui_framework_channel).await?;
    systems.log_component("apps/playground-editor", playground_systems_logic::LogLevel::Info,
        format!("✓ UiFrameworkPlugin registered with channel {}", ui_framework_channel)).await;

    // 2. Editor Core - Text editing with vim mode  
    let editor_core_channel = systems.register_plugin("editor-core").await?;
    world.write().await.register_plugin_channel("editor-core".to_string(), editor_core_channel).await?;
    eprintln!("[EDITOR] ✓ EditorCorePlugin registered with channel {}", editor_core_channel);

    // 3. File Browser - File navigation
    let file_browser_channel = systems.register_plugin("file-browser").await?;
    world.write().await.register_plugin_channel("file-browser".to_string(), file_browser_channel).await?;
    eprintln!("[EDITOR] ✓ FileBrowserPlugin registered with channel {}", file_browser_channel);

    // 4. Terminal - Termux integration
    let terminal_channel = systems.register_plugin("terminal").await?;
    world.write().await.register_plugin_channel("terminal".to_string(), terminal_channel).await?;
    eprintln!("[EDITOR] ✓ TerminalPlugin registered with channel {}", terminal_channel);

    // 5. LSP Client - Language server protocol
    let lsp_client_channel = systems.register_plugin("lsp-client").await?;
    world.write().await.register_plugin_channel("lsp-client".to_string(), lsp_client_channel).await?;
    eprintln!("[EDITOR] ✓ LspClientPlugin registered with channel {}", lsp_client_channel);

    // 6. Debugger - Debug support
    let debugger_channel = systems.register_plugin("debugger").await?;
    world.write().await.register_plugin_channel("debugger".to_string(), debugger_channel).await?;
    eprintln!("[EDITOR] ✓ DebuggerPlugin registered with channel {}", debugger_channel);

    // 7. Chat Assistant - MCP/LLM integration
    let chat_assistant_channel = systems.register_plugin("chat-assistant").await?;
    world.write().await.register_plugin_channel("chat-assistant".to_string(), chat_assistant_channel).await?;
    eprintln!("[EDITOR] ✓ ChatAssistantPlugin registered with channel {}", chat_assistant_channel);

    // 8. Version Control - Git integration
    let version_control_channel = systems.register_plugin("version-control").await?;
    world.write().await.register_plugin_channel("version-control".to_string(), version_control_channel).await?;
    eprintln!("[EDITOR] ✓ VersionControlPlugin registered with channel {}", version_control_channel);

    // 9. Theme Manager - UI theming
    let theme_manager_channel = systems.register_plugin("theme-manager").await?;
    world.write().await.register_plugin_channel("theme-manager".to_string(), theme_manager_channel).await?;
    eprintln!("[EDITOR] ✓ ThemeManagerPlugin registered with channel {}", theme_manager_channel);

    eprintln!("[EDITOR] Phase 1 complete: All IDE plugins registered with dynamic channels!");

    // Phase 2: CORE INITIALIZATION - Initialize core engine systems
    // Now that all plugins are registered, the NetworkingSystem can build
    // a complete channel manifest
    eprintln!("[EDITOR] Phase 2: Initializing core engine systems...");
    systems.initialize_all().await?;
    eprintln!("[EDITOR] Phase 2 complete: Core engine systems initialized!");

    // Phase 3: PLUGIN INITIALIZATION - Start plugins as independent tasks
    // Now that NetworkingSystem is ready, plugins can perform network operations
    eprintln!("[EDITOR] Phase 3: Starting plugin tasks...");
    
    // Convert Shared<World> to Handle<World> for plugins (external reference)
    let world_handle: Handle<World> = handle(world.read().await.clone());
    
    // 1. UI Framework Plugin
    {
        let mut plugin = UiFrameworkPlugin::new(systems.clone());
        plugin.initialize(&*world_handle).await?;
        eprintln!("[EDITOR] ✓ UiFrameworkPlugin initialized");
        
        let plugin_world = world_handle.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Err(e) = plugin.run(&*plugin_world, 0.016).await {
                    // Note: Can't use systems.log_component here as systems not available in spawned task
                    eprintln!("[UiFrameworkPlugin] Error: {}", e);
                }
            }
        });
    }
    
    // 2. Editor Core Plugin
    {
        let plugin_world = world_handle.clone();
        let mut plugin = EditorCorePlugin::new(systems.clone());
        plugin.initialize(&*world_handle).await?;
        eprintln!("[EDITOR] ✓ EditorCorePlugin initialized");
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Err(e) = plugin.run(&*plugin_world, 0.016).await {
                    eprintln!("[EditorCorePlugin] Error: {}", e);
                }
            }
        });
    }
    
    // 3. File Browser Plugin
    {
        let plugin_world = world_handle.clone();
        let mut plugin = FileBrowserPlugin::new(systems.clone());
        plugin.initialize(&*world_handle).await?;
        eprintln!("[EDITOR] ✓ FileBrowserPlugin initialized");
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Err(e) = plugin.run(&*plugin_world, 0.016).await {
                    eprintln!("[FileBrowserPlugin] Error: {}", e);
                }
            }
        });
    }
    
    // 4. Terminal Plugin
    {
        let plugin_world = world_handle.clone();
        let mut plugin = TerminalPlugin::new(systems.clone());
        plugin.initialize(&*world_handle).await?;
        eprintln!("[EDITOR] ✓ TerminalPlugin initialized");
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Err(e) = plugin.run(&*plugin_world, 0.016).await {
                    eprintln!("[TerminalPlugin] Error: {}", e);
                }
            }
        });
    }
    
    // 5. LSP Client Plugin
    {
        let plugin_world = world_handle.clone();
        let mut plugin = LspClientPlugin::new(systems.clone());
        plugin.initialize(&*world_handle).await?;
        eprintln!("[EDITOR] ✓ LspClientPlugin initialized");
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Err(e) = plugin.run(&*plugin_world, 0.016).await {
                    eprintln!("[LspClientPlugin] Error: {}", e);
                }
            }
        });
    }
    
    // 6. Debugger Plugin
    {
        let plugin_world = world_handle.clone();
        let mut plugin = DebuggerPlugin::new(systems.clone());
        plugin.initialize(&*world_handle).await?;
        eprintln!("[EDITOR] ✓ DebuggerPlugin initialized");
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Err(e) = plugin.run(&*plugin_world, 0.016).await {
                    eprintln!("[DebuggerPlugin] Error: {}", e);
                }
            }
        });
    }
    
    // 7. Chat Assistant Plugin
    {
        let plugin_world = world_handle.clone();
        let mut plugin = ChatAssistantPlugin::new(systems.clone());
        plugin.initialize(&*world_handle).await?;
        eprintln!("[EDITOR] ✓ ChatAssistantPlugin initialized");
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Err(e) = plugin.run(&*plugin_world, 0.016).await {
                    eprintln!("[ChatAssistantPlugin] Error: {}", e);
                }
            }
        });
    }
    
    // 8. Version Control Plugin
    {
        let plugin_world = world_handle.clone();
        let mut plugin = VersionControlPlugin::new(systems.clone());
        plugin.initialize(&*world_handle).await?;
        eprintln!("[EDITOR] ✓ VersionControlPlugin initialized");
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Err(e) = plugin.run(&*plugin_world, 0.016).await {
                    eprintln!("[VersionControlPlugin] Error: {}", e);
                }
            }
        });
    }
    
    // 9. Theme Manager Plugin
    {
        let plugin_world = world_handle.clone();
        let mut plugin = ThemeManagerPlugin::new(systems.clone());
        plugin.initialize(&*world_handle).await?;
        eprintln!("[EDITOR] ✓ ThemeManagerPlugin initialized");
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Err(e) = plugin.run(&*plugin_world, 0.016).await {
                    eprintln!("[ThemeManagerPlugin] Error: {}", e);
                }
            }
        });
    }
    
    eprintln!("[EDITOR] Phase 3 complete: All plugin tasks started!");
    
    // Display the dynamically allocated channels
    eprintln!("[EDITOR] Dynamic channel allocations:");
    let manifest = systems.get_channel_manifest().await;
    for (name, channel) in &manifest.channels {
        eprintln!("  - {}: {}", channel, name);
    }
    
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