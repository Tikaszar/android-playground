use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{LogicResult, LogicError};
use crate::world::World;

// Import other systems
use playground_systems_networking::NetworkingSystem;
use playground_systems_ui::UiSystem;
// use playground_systems_rendering::RenderingSystem; // Browser-side only
// use playground_systems_physics::PhysicsSystem; // Not yet implemented

/// Manages all system instances for the engine
pub struct SystemsManager {
    world: Arc<RwLock<World>>,
    pub networking: Arc<RwLock<NetworkingSystem>>,
    pub ui: Arc<RwLock<UiSystem>>,
    // Note: RenderingSystem needs a renderer type, but WebGL isn't thread-safe
    // For now, we'll skip rendering in the server-side logic
    // pub rendering: Arc<RwLock<RenderingSystem<SomeRenderer>>>,
    // pub physics: Arc<RwLock<PhysicsSystem>>,
}

impl SystemsManager {
    /// Create a new SystemsManager with reference to the World
    pub async fn new(world: Arc<RwLock<World>>) -> LogicResult<Self> {
        let networking = NetworkingSystem::new().await
            .map_err(|e| LogicError::InitializationFailed(format!("NetworkingSystem: {}", e)))?;
        
        Ok(Self {
            world,
            networking: Arc::new(RwLock::new(networking)),
            ui: Arc::new(RwLock::new(UiSystem::new())),
        })
    }
    
    /// Initialize all systems
    pub async fn initialize_all(&self) -> LogicResult<()> {
        // Initialize NetworkingSystem
        // It will start core/server internally if not already running
        let mut networking = self.networking.write().await;
        networking.initialize(None).await
            .map_err(|e| LogicError::InitializationFailed(format!("NetworkingSystem: {}", e)))?;
        
        // After server is running, we can log to its dashboard
        if let Some(dashboard) = networking.get_dashboard().await {
            use playground_core_server::dashboard::LogLevel;
            
            dashboard.log(
                LogLevel::Info,
                "Initializing all engine systems...".to_string(),
                None
            ).await;
            
            dashboard.log(
                LogLevel::Info,
                "✓ NetworkingSystem initialized (started core/server internally)".to_string(),
                None
            ).await;
            
            // Initialize UiSystem
            dashboard.log(
                LogLevel::Info,
                "✓ UiSystem initialized".to_string(),
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
        }
        
        Ok(())
    }
    
    /// Get reference to NetworkingSystem
    pub fn networking(&self) -> Arc<RwLock<NetworkingSystem>> {
        self.networking.clone()
    }
    
    /// Get reference to UiSystem
    pub fn ui(&self) -> Arc<RwLock<UiSystem>> {
        self.ui.clone()
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
    
}