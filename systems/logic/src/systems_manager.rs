use playground_core_types::{Shared, shared};
use crate::error::{LogicResult, LogicError};
use crate::world::World;

// Import other systems
use playground_systems_networking::NetworkingSystem;
use playground_systems_ui::UiSystem;
// use playground_systems_rendering::RenderingSystem; // Browser-side only
// use playground_systems_physics::PhysicsSystem; // Not yet implemented

/// Manages all system instances for the engine
pub struct SystemsManager {
    world: Shared<World>,
    pub networking: Shared<NetworkingSystem>,
    pub ui: Shared<UiSystem>,
    renderer: Option<Shared<Box<dyn playground_core_rendering::Renderer>>>,
    // pub physics: Shared<PhysicsSystem>, // Not yet implemented
}

impl SystemsManager {
    /// Create a new SystemsManager with reference to the World
    pub async fn new(world: Shared<World>) -> LogicResult<Self> {
        let networking = NetworkingSystem::new().await
            .map_err(|e| LogicError::InitializationFailed(format!("NetworkingSystem: {}", e)))?;
        
        Ok(Self {
            world,
            networking: shared(networking),
            ui: shared(UiSystem::new()),
            renderer: None,
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
        let dashboard = if let Some(dashboard) = networking.get_dashboard().await {
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
            
            Some(dashboard)
        } else {
            None
        };
        
        // Initialize UiSystem
        let mut ui = self.ui.write().await;
        ui.initialize().await
            .map_err(|e| LogicError::InitializationFailed(format!("UiSystem: {}", e)))?;
        
        // TODO: Set up channel connection for UiSystem
        // This would need access to the WebSocketState from core/server
        // For now, the UI system won't be able to send render commands
        
        if let Some(ref dashboard) = dashboard {
            use playground_core_server::dashboard::LogLevel;
            
            dashboard.log(
                LogLevel::Info,
                "✓ UiSystem initialized (headless mode)".to_string(),
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
    pub fn networking(&self) -> Shared<NetworkingSystem> {
        self.networking.clone()
    }
    
    /// Get reference to UiSystem
    pub fn ui(&self) -> Shared<UiSystem> {
        self.ui.clone()
    }
    
    /// Set the renderer for the UI system
    pub async fn set_renderer(&mut self, renderer: Box<dyn playground_core_rendering::Renderer>) {
        self.renderer = Some(shared(renderer));
    }
    
    /// Render a frame using the current renderer
    pub async fn render_frame(&self) -> LogicResult<()> {
        let mut ui = self.ui.write().await;
        let batch = ui.render().await
            .map_err(|e| LogicError::SystemError(format!("UI render failed: {}", e)))?;
        
        if let Some(ref renderer) = self.renderer {
            let mut r = renderer.write().await;
            r.begin_frame().await
                .map_err(|e| LogicError::SystemError(format!("Begin frame failed: {}", e)))?;
            r.execute_commands(&batch).await
                .map_err(|e| LogicError::SystemError(format!("Execute commands failed: {}", e)))?;
            r.end_frame().await
                .map_err(|e| LogicError::SystemError(format!("End frame failed: {}", e)))?;
            r.present().await
                .map_err(|e| LogicError::SystemError(format!("Present failed: {}", e)))?;
        }
        
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
        let net = self.networking.read().await;
        net.register_mcp_tool(name.clone(), description, input_schema, handler_channel).await
            .map_err(|e| LogicError::SystemError(format!("Failed to register MCP tool '{}': {}", name, e)))?;
        Ok(())
    }
    
}