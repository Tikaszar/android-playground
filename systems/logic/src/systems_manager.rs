use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{LogicResult, LogicError};

// Import other systems
use playground_networking::NetworkingSystem;
use playground_ui::UiSystem;
// use playground_rendering::RenderingSystem; // Browser-side only
// use playground_physics::PhysicsSystem; // Not yet implemented

/// Manages all system instances for the engine
pub struct SystemsManager {
    pub networking: Arc<RwLock<NetworkingSystem>>,
    pub ui: Arc<RwLock<UiSystem>>,
    // Note: RenderingSystem needs a renderer type, but WebGL isn't thread-safe
    // For now, we'll skip rendering in the server-side logic
    // pub rendering: Arc<RwLock<RenderingSystem<SomeRenderer>>>,
    // pub physics: Arc<RwLock<PhysicsSystem>>,
}

impl SystemsManager {
    /// Initialize all systems
    /// This is called by systems/logic when the ECS is created
    pub async fn initialize() -> LogicResult<Self> {
        tracing::info!("Initializing all engine systems...");
        
        // Initialize NetworkingSystem
        // It will create and manage core/server connection using core/ecs
        let mut networking = NetworkingSystem::new().await
            .map_err(|e| LogicError::InitializationFailed(format!("NetworkingSystem: {}", e)))?;
        
        // Connect to core/server WebSocket
        networking.initialize(None).await
            .map_err(|e| LogicError::InitializationFailed(format!("NetworkingSystem connect: {}", e)))?;
        
        tracing::info!("✓ NetworkingSystem initialized and connected to core/server");
        
        // Initialize UiSystem
        // It uses core/ecs internally for state management
        let ui = UiSystem::new();
        tracing::info!("✓ UiSystem initialized");
        
        // RenderingSystem initialization skipped for now
        // WebGL renderer isn't thread-safe and can't be used server-side
        // The browser client will handle rendering directly
        tracing::info!("✓ RenderingSystem skipped (browser-side only)");
        
        // Initialize PhysicsSystem (when implemented)
        // let physics = PhysicsSystem::new();
        // tracing::info!("✓ PhysicsSystem initialized");
        
        Ok(Self {
            networking: Arc::new(RwLock::new(networking)),
            ui: Arc::new(RwLock::new(ui)),
            // rendering: Arc::new(RwLock::new(rendering)),
            // physics: Arc::new(RwLock::new(physics)),
        })
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
        let net = self.networking.read().await;
        net.register_mcp_tool(name.clone(), description, input_schema, handler_channel).await
            .map_err(|e| LogicError::SystemError(format!("Failed to register MCP tool '{}': {}", name, e)))?;
        Ok(())
    }
    
    /// Register plugin channels with the networking system
    pub async fn register_plugin_channels(&self, plugin_name: &str, base_channel: u16, count: u16) -> LogicResult<()> {
        let net = self.networking.read().await;
        
        // Register the plugin once to get its base channel
        let registered_channel = net.register_plugin(plugin_name).await
            .map_err(|e| LogicError::SystemError(format!("Failed to register plugin '{}': {}", plugin_name, e)))?;
        
        tracing::info!("Registered plugin '{}' on channel {} (requested base {}, count {})", 
            plugin_name, registered_channel, base_channel, count);
        
        // Note: The actual channel allocation is handled by the networking system
        // Plugins get a single channel ID and can use sub-channels internally
        
        Ok(())
    }
}