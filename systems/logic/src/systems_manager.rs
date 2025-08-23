use playground_core_types::{Shared, shared};
use crate::error::{LogicResult, LogicError};
use crate::world::World;
use crate::ui_interface::UiInterface;
use crate::rendering_interface::{RenderingInterface, RendererWrapper};

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
        
        // Register UI system with networking and get its channel
        let ui_channel = networking.register_system_channel("ui", 10).await
            .map_err(|e| LogicError::InitializationFailed(format!("Failed to register UI channel: {}", e)))?;
        
        // Store networking reference in UI system so it can send render commands
        ui.set_networking_system(self.networking.clone());
        
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
    pub async fn set_renderer(&mut self, renderer: Box<dyn playground_core_rendering::Renderer>) {
        self.renderer = Some(shared(renderer));
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
        let net = self.networking.read().await;
        net.register_mcp_tool(name.clone(), description, input_schema, handler_channel).await
            .map_err(|e| LogicError::SystemError(format!("Failed to register MCP tool '{}': {}", name, e)))?;
        Ok(())
    }
    
    /// Get UI interface for plugins to interact with UI system
    pub fn ui_interface(&self) -> UiInterface {
        UiInterface::new(self.ui.clone())
    }
    
    /// Get rendering interface if a renderer is set
    pub fn rendering_interface(&self) -> Option<Box<dyn RenderingInterface>> {
        self.renderer.as_ref().map(|r| {
            Box::new(RendererWrapper::new(r.clone())) as Box<dyn RenderingInterface>
        })
    }
    
    /// Get the shared World for plugins to use
    pub fn world(&self) -> Shared<World> {
        self.world.clone()
    }
    
    /// Log a message to the dashboard
    pub async fn log(&self, level: &str, message: String) {
        if let Some(dashboard) = self.networking.read().await.get_dashboard().await {
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
    
    /// Start the render loop at 60fps
    pub async fn start_render_loop(&self) -> LogicResult<()> {
        use std::time::{Duration, Instant};
        use tokio::time::interval_at;
        
        // TODO: Get channel manager from networking system to connect UI
        // The networking system needs to provide access to its channel manager
        // For now, the UI will generate render commands but not send them
        
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