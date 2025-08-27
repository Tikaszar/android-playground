use playground_core_types::{Shared, shared, Handle, handle};
use crate::error::{LogicResult, LogicError};
use crate::world::World;
use crate::ui_interface::UiInterface;
use crate::rendering_interface::{RendererWrapper, RendererData};
use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

// Import other systems
use playground_systems_networking::NetworkingSystem;
use playground_systems_ui::UiSystem;
// use playground_systems_rendering::RenderingSystem; // Browser-side only
// use playground_systems_physics::PhysicsSystem; // Not yet implemented

/// Type of channel registration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChannelType {
    System,   // Core systems (ui, networking, etc.)
    Plugin,   // Plugin channels
    Session,  // Dynamic session channels (like MCP sessions)
}

/// Metadata about a registered channel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelMetadata {
    pub name: String,
    pub channel_id: u16,
    pub registered_at: SystemTime,
    pub channel_type: ChannelType,
    pub description: Option<String>,
}

/// Channel manifest sent to browser for discovery
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelManifest {
    pub version: u32,
    pub channels: HashMap<String, u16>,
    pub metadata: HashMap<u16, ChannelMetadata>,
}

/// Registry for dynamic channel allocation
pub struct ChannelRegistry {
    // Map of name -> channel ID
    channels: HashMap<String, u16>,
    // Reverse map for lookups
    channel_names: HashMap<u16, String>,
    // Next available channel ID (starts at 1, since 0 is control)
    next_channel_id: u16,
    // Optional metadata about each channel
    channel_metadata: HashMap<u16, ChannelMetadata>,
}

impl ChannelRegistry {
    /// Create a new channel registry
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            channel_names: HashMap::new(),
            next_channel_id: 1, // Start at 1, channel 0 is reserved for control
            channel_metadata: HashMap::new(),
        }
    }
    
    /// Allocate a new channel for a system or plugin
    pub fn allocate_channel(&mut self, name: String, channel_type: ChannelType) -> LogicResult<u16> {
        // Check if already registered
        if let Some(&channel_id) = self.channels.get(&name) {
            return Ok(channel_id);
        }
        
        // Allocate next available channel
        let channel_id = self.next_channel_id;
        self.next_channel_id += 1;
        
        // Store mapping
        self.channels.insert(name.clone(), channel_id);
        self.channel_names.insert(channel_id, name.clone());
        
        // Store metadata
        let metadata = ChannelMetadata {
            name: name.clone(),
            channel_id,
            registered_at: SystemTime::now(),
            channel_type,
            description: None,
        };
        self.channel_metadata.insert(channel_id, metadata);
        
        Ok(channel_id)
    }
    
    /// Get channel ID by name
    pub fn get_channel(&self, name: &str) -> Option<u16> {
        self.channels.get(name).copied()
    }
    
    /// Get channel name by ID
    pub fn get_channel_name(&self, channel_id: u16) -> Option<String> {
        self.channel_names.get(&channel_id).cloned()
    }
    
    /// Generate channel manifest for discovery
    pub fn generate_manifest(&self) -> ChannelManifest {
        ChannelManifest {
            version: 1,
            channels: self.channels.clone(),
            metadata: self.channel_metadata.clone(),
        }
    }
}

/// Manages all system instances for the engine
pub struct SystemsManager {
    world: Shared<World>,
    pub networking: Shared<NetworkingSystem>,  // NetworkingSystem needs mutable methods
    pub ui: Shared<UiSystem>,
    renderer: Option<RendererWrapper>,
    renderer_data: Option<Shared<RendererData>>,
    // pub physics: Shared<PhysicsSystem>, // Not yet implemented
    
    // NEW: Dynamic channel registry
    channel_registry: Shared<ChannelRegistry>,
}

impl SystemsManager {
    /// Create a new SystemsManager with reference to the World
    pub async fn new(world: Shared<World>) -> LogicResult<Self> {
        let networking = NetworkingSystem::new().await
            .map_err(|e| LogicError::InitializationFailed(format!("NetworkingSystem: {}", e)))?;
        
        Ok(Self {
            world,
            networking: shared(networking),  // Use shared() for NetworkingSystem
            ui: shared(UiSystem::new()),
            renderer: None,
            renderer_data: None,
            channel_registry: shared(ChannelRegistry::new()),
        })
    }
    
    /// Initialize all systems
    pub async fn initialize_all(&self) -> LogicResult<()> {
        // Set up the channel manifest callback before initializing networking
        {
            let mut networking = self.networking.write().await;
            
            // Create a weak reference to avoid circular dependency
            let channel_registry = self.channel_registry.clone();
            
            // Set the callback that will provide channel manifest
            use playground_systems_networking::ChannelManifestCallback;
            let callback: ChannelManifestCallback = Box::new(move || {
                let registry = channel_registry.clone();
                Box::pin(async move {
                    let reg = registry.read().await;
                    let manifest = reg.generate_manifest();
                    bincode::serialize(&manifest)
                        .map_err(|e| format!("Failed to serialize manifest: {}", e))
                })
            });
            
            networking.set_channel_manifest_callback(callback);
        }
        
        // Initialize NetworkingSystem
        // It will start core/server internally if not already running
        let mut networking = self.networking.write().await;
        networking.initialize(None).await
            .map_err(|e| LogicError::InitializationFailed(format!("NetworkingSystem: {}", e)))?;
        
        // After server is running, we can log to its dashboard
        let dashboard = if let Some(dashboard) = networking.get_dashboard().await {
            use playground_core_server::dashboard::LogLevel;
            
            dashboard.log(
                LogLevel::Info,
                "Initializing all engine systems with dynamic channels...".to_string(),
                None
            ).await;
            
            dashboard.log(
                LogLevel::Info,
                "✓ NetworkingSystem initialized (started core/server internally)".to_string(),
                None
            ).await;
            
            Some(dashboard)
        } else {
            None
        };
        
        // Initialize UiSystem
        let mut ui = self.ui.write().await;
        ui.initialize().await
            .map_err(|e| LogicError::InitializationFailed(format!("UiSystem: {}", e)))?;
        
        // Dynamically allocate channel for UI system
        let ui_channel = {
            let mut registry = self.channel_registry.write().await;
            registry.allocate_channel("ui".to_string(), ChannelType::System)?
        };
        
        // Register UI system with networking using dynamic channel
        let _ui_channel = networking.register_system_channel("ui", ui_channel).await
            .map_err(|e| LogicError::InitializationFailed(format!("Failed to register UI channel: {}", e)))?;
        
        // Drop the networking write lock before setting it in UI
        drop(networking);
        
        // Store networking reference in UI system so it can send render commands
        // UiSystem expects Handle but we have Shared - need to fix UiSystem
        ui.set_networking_system_shared(self.networking.clone());
        
        if let Some(ref dashboard) = dashboard {
            use playground_core_server::dashboard::LogLevel;
            
            dashboard.log(
                LogLevel::Info,
                format!("✓ UiSystem initialized on dynamic channel {}", ui_channel),
                None
            ).await;
            
            // RenderingSystem initialization skipped for now
            dashboard.log(
                LogLevel::Info,
                "✓ RenderingSystem skipped (browser-side only)".to_string(),
                None
            ).await;
            
            dashboard.log(
                LogLevel::Info,
                "✓ All systems initialized successfully".to_string(),
                None
            ).await;
            
            dashboard.log(
                LogLevel::Info,
                "Starting 60fps render loop...".to_string(),
                None
            ).await;
        }
        
        // Start the render loop at 60fps
        self.start_render_loop().await?;
        
        if let Some(ref dashboard) = dashboard {
            use playground_core_server::dashboard::LogLevel;
            dashboard.log(
                LogLevel::Info,
                "✓ Render loop started".to_string(),
                None
            ).await;
        }
        
        Ok(())
    }
    
    /// Get reference to NetworkingSystem
    pub fn networking(&self) -> Shared<NetworkingSystem> {
        self.networking.clone()
    }
    
    /// Get reference to UiSystem
    pub fn ui(&self) -> Shared<UiSystem> {
        self.ui.clone()
    }
    
    /// Set the renderer for the UI system
    pub async fn set_renderer(&mut self, renderer_type: String, renderer_channel: u16) {
        let renderer_data = shared(RendererData::new(renderer_type.clone()));
        self.renderer = Some(RendererWrapper::new(renderer_type, renderer_channel));
        self.renderer_data = Some(renderer_data);
    }
    
    /// Render a frame using the current renderer
    pub async fn render_frame(&self) -> LogicResult<()> {
        let mut ui = self.ui.write().await;
        // render() now sends commands directly via channel 10
        ui.render().await
            .map_err(|e| LogicError::SystemError(format!("UI render failed: {}", e)))?;
        
        // If we have a local renderer (for testing), we could use it here
        // but normally rendering happens in the browser via WebGL
        
        Ok(())
    }
    
    /// Register an MCP tool for a plugin or app
    /// This wraps the networking system's MCP tool registration
    pub async fn register_mcp_tool(
        &self,
        name: String,
        description: String,
        input_schema: serde_json::Value,
        handler_channel: u16,
    ) -> LogicResult<()> {
        let networking = self.networking.read().await;
        networking.register_mcp_tool(name.clone(), description, input_schema, handler_channel).await
            .map_err(|e| LogicError::SystemError(format!("Failed to register MCP tool '{}': {}", name, e)))?;
        Ok(())
    }
    
    /// Get UI interface for plugins to interact with UI system
    pub fn ui_interface(&self) -> UiInterface {
        UiInterface::new(self.ui.clone())
    }
    
    /// Get rendering interface if a renderer is set
    pub fn rendering_interface(&self) -> Option<RendererWrapper> {
        self.renderer.clone()
    }
    
    /// Get the shared World for plugins to use
    pub fn world(&self) -> Shared<World> {
        self.world.clone()
    }
    
    /// Log a message to the dashboard
    pub async fn log(&self, level: &str, message: String) {
        let networking = self.networking.read().await;
        if let Some(dashboard) = networking.get_dashboard().await {
            use playground_core_server::dashboard::LogLevel;
            let log_level = match level {
                "error" | "Error" => LogLevel::Error,
                "warn" | "Warning" => LogLevel::Warning,
                "info" | "Info" => LogLevel::Info,
                "debug" | "Debug" => LogLevel::Debug,
                _ => LogLevel::Info,
            };
            dashboard.log(log_level, message, None).await;
        }
    }
    
    /// Register a system and get its dynamically allocated channel
    pub async fn register_system(&self, name: &str) -> LogicResult<u16> {
        let mut registry = self.channel_registry.write().await;
        registry.allocate_channel(name.to_string(), ChannelType::System)
    }
    
    /// Register a plugin and get its dynamically allocated channel
    pub async fn register_plugin(&self, name: &str) -> LogicResult<u16> {
        let mut registry = self.channel_registry.write().await;
        registry.allocate_channel(name.to_string(), ChannelType::Plugin)
    }
    
    /// Get channel manifest for browser discovery
    pub async fn get_channel_manifest(&self) -> ChannelManifest {
        let registry = self.channel_registry.read().await;
        registry.generate_manifest()
    }
    
    /// Get serialized channel manifest for sending to browser
    pub async fn get_channel_manifest_bytes(&self) -> LogicResult<Vec<u8>> {
        let manifest = self.get_channel_manifest().await;
        bincode::serialize(&manifest)
            .map_err(|e| LogicError::SystemError(format!("Failed to serialize manifest: {}", e)))
    }
    
    /// Get channel ID by name
    pub async fn get_channel_by_name(&self, name: &str) -> Option<u16> {
        let registry = self.channel_registry.read().await;
        registry.get_channel(name)
    }
    
    /// Start the render loop at 60fps
    pub async fn start_render_loop(&self) -> LogicResult<()> {
        use std::time::{Duration, Instant};
        use tokio::time::interval_at;
        
        // The UI system has access to networking system and sends commands via channel 10
        // This is already configured in initialize_all()
        
        // Create 60fps interval (16.67ms)
        let frame_duration = Duration::from_millis(16);
        let start = Instant::now() + frame_duration;
        let mut interval = interval_at(start.into(), frame_duration);
        
        // Clone references for the spawned task
        let ui_system = self.ui.clone();
        
        // Spawn render loop task
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                // Render the UI
                let mut ui = ui_system.write().await;
                if let Err(e) = ui.render().await {
                    tracing::warn!("UI render error: {}", e);
                }
            }
        });
        
        Ok(())
    }
}