use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{LogicResult, LogicError};

// Import other systems
use playground_networking::NetworkingSystem;
use playground_ui::UiSystem;
use playground_rendering::RenderingSystem;
// use playground_physics::PhysicsSystem; // Not yet implemented

/// Manages all system instances for the engine
pub struct SystemsManager {
    pub networking: Arc<RwLock<NetworkingSystem>>,
    pub ui: Arc<RwLock<UiSystem>>,
    pub rendering: Arc<RwLock<RenderingSystem>>,
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
        
        // Initialize RenderingSystem
        // It uses core/ecs internally for resource tracking
        let rendering = RenderingSystem::new();
        tracing::info!("✓ RenderingSystem initialized");
        
        // Initialize PhysicsSystem (when implemented)
        // let physics = PhysicsSystem::new();
        // tracing::info!("✓ PhysicsSystem initialized");
        
        Ok(Self {
            networking: Arc::new(RwLock::new(networking)),
            ui: Arc::new(RwLock::new(ui)),
            rendering: Arc::new(RwLock::new(rendering)),
            // physics: Arc::new(RwLock::new(physics)),
        })
    }
    
    /// Register plugin channels with the networking system
    pub async fn register_plugin_channels(&self, plugin_name: &str, base_channel: u16, count: u16) -> LogicResult<()> {
        let mut net = self.networking.write().await;
        
        for i in 0..count {
            let channel = base_channel + i;
            net.register_channel(channel, format!("{}-{}", plugin_name, i)).await
                .map_err(|e| LogicError::SystemError(format!("Failed to register channel {}: {}", channel, e)))?;
        }
        
        tracing::info!("Registered {} channels for plugin '{}' ({}..{})", 
            count, plugin_name, base_channel, base_channel + count - 1);
        
        Ok(())
    }
}