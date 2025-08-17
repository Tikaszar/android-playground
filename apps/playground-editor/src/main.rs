mod messages;
mod message_bus;
mod layout;

use anyhow::Result;
use dashmap::DashMap;
use playground_plugin::{Plugin, PluginLoader};
use playground_server::{ChannelManager, FrameBatcher};
use playground_ui::UiSystem;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn};
use uuid::Uuid;

use crate::message_bus::MessageBus;
use crate::layout::IdeLayout;
use crate::messages::{PluginMessage, MessageEnvelope};

/// The Playground Editor IDE application.
/// Coordinates multiple editor plugins to provide a complete development environment.
pub struct PlaygroundEditorApp {
    /// Loaded plugins mapped by their IDs
    plugins: Arc<DashMap<Uuid, Box<dyn Plugin>>>,
    /// Plugin loader for dynamic loading
    plugin_loader: PluginLoader,
    /// UI system instance
    ui_system: Arc<RwLock<UiSystem>>,
    /// Channel manager for WebSocket communication
    channel_manager: Arc<ChannelManager>,
    /// Frame batcher for packet batching
    frame_batcher: Arc<FrameBatcher>,
    /// Message bus for inter-plugin communication
    message_bus: Arc<MessageBus>,
    /// IDE layout
    layout: Arc<RwLock<IdeLayout>>,
    /// Application configuration
    config: AppConfig,
}

#[derive(Clone)]
pub struct AppConfig {
    /// Plugin directory path
    pub plugin_dir: String,
    /// Auto-load plugins on startup
    pub auto_load_plugins: bool,
    /// WebSocket server port
    pub server_port: u16,
    /// Enable hot-reload
    pub hot_reload: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            plugin_dir: "./plugins".to_string(),
            auto_load_plugins: true,
            server_port: 3000,
            hot_reload: true,
        }
    }
}

impl PlaygroundEditorApp {
    pub fn new(config: AppConfig) -> Result<Self> {
        let channel_manager = Arc::new(ChannelManager::new());
        let frame_batcher = Arc::new(FrameBatcher::new(2000, 60)); // 2000 channels, 60fps
        let ui_system = Arc::new(RwLock::new(UiSystem::new()));
        let (message_bus, _broadcast_receiver) = MessageBus::new();
        let layout = Arc::new(RwLock::new(IdeLayout::new()));
        
        Ok(Self {
            plugins: Arc::new(DashMap::new()),
            plugin_loader: PluginLoader::new(),
            ui_system,
            channel_manager,
            frame_batcher,
            message_bus: Arc::new(message_bus),
            layout,
            config,
        })
    }

    /// Load all IDE plugins
    pub async fn load_plugins(&self) -> Result<()> {
        info!("Loading IDE plugins...");
        
        // List of IDE plugins to load (channels 1000-1079)
        let ide_plugins = vec![
            ("editor-core", 1000),
            ("file-browser", 1010),
            ("terminal", 1020),
            ("lsp-client", 1030),
            ("debugger", 1040),
            ("chat-assistant", 1050),
            ("version-control", 1060),
            ("theme-manager", 1070),
        ];
        
        for (plugin_name, base_channel) in ide_plugins {
            match self.load_plugin(plugin_name, base_channel).await {
                Ok(id) => info!("Loaded plugin '{}' with ID {}", plugin_name, id),
                Err(e) => warn!("Failed to load plugin '{}': {}", plugin_name, e),
            }
        }
        
        Ok(())
    }

    /// Load a single plugin
    async fn load_plugin(&self, name: &str, base_channel: u16) -> Result<Uuid> {
        let plugin_path = format!("{}/{}/lib{}.so", self.config.plugin_dir, name, name.replace("-", "_"));
        
        // For now, we'll skip actual loading since plugins aren't compiled yet
        // In production, this would use plugin_loader.load(&plugin_path)
        
        info!("Would load plugin from: {}", plugin_path);
        
        // Register plugin channels
        for i in 0..10 {
            self.channel_manager.register_channel(
                base_channel + i,
                format!("{}-{}", name, i),
            );
        }
        
        Ok(Uuid::new_v4())
    }

    /// Main application loop
    pub async fn run(&self) -> Result<()> {
        info!("Starting Playground Editor IDE...");
        
        // Load plugins if auto-load is enabled
        if self.config.auto_load_plugins {
            self.load_plugins().await?;
        }
        
        // Start the UI system
        info!("Initializing UI system...");
        {
            let mut ui = self.ui_system.write().await;
            // UI system initialization would happen here
        }
        
        // Start the server
        info!("Starting server on port {}...", self.config.server_port);
        
        // Main application loop
        loop {
            // Process plugin updates
            for plugin in self.plugins.iter() {
                // Call plugin update methods
                // plugin.update(delta_time);
            }
            
            // Process frame batching
            // In a real implementation, we would process batches per channel
            // For now, just a placeholder
            
            // Sleep for frame time (16ms for 60fps)
            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
        }
    }

    /// Shutdown the application
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Playground Editor...");
        
        // Unload all plugins
        for plugin in self.plugins.iter() {
            let (id, _) = plugin.pair();
            info!("Unloading plugin {}", id);
            // plugin.on_unload();
        }
        
        self.plugins.clear();
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();
    
    info!("Playground Editor IDE starting...");
    
    // Load configuration (could be from file in future)
    let config = AppConfig::default();
    
    // Create and run the application
    let app = PlaygroundEditorApp::new(config)?;
    
    // Set up shutdown handler
    let app_clone = Arc::new(app);
    let shutdown_app = app_clone.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_app.shutdown().await.ok();
        std::process::exit(0);
    });
    
    // Run the application
    app_clone.run().await?;
    
    Ok(())
}