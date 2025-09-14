//! Systems/Logic - The ONLY Public API Gateway
//! 
//! This is a STATELESS API gateway that provides the ONLY interface
//! that plugins and apps can use to interact with the engine.
//! 
//! ARCHITECTURE RULES:
//! - NO implementation code - only API functions that forward to command processors
//! - NO state - this is purely a gateway layer
//! - ONLY uses core/* contracts, NEVER imports other systems/*
//! - ALL functionality exposed through clean public APIs
//! - Hides all implementation details from plugins/apps

// API modules - each provides clean functions for a specific domain
pub mod ecs_api;
pub mod networking_api;
pub mod console_api;
pub mod client_api;
pub mod ui_api;
pub mod rendering_api;

// Re-export all API functions for convenience
pub use ecs_api::*;
pub use networking_api::*;
pub use console_api::*;
pub use client_api::*;
pub use ui_api::*;
pub use rendering_api::*;

// Public types that plugins/apps need
// These are the ONLY types exposed to the outside world

// Re-export Handle and Shared for plugins/apps
pub use playground_core_types::{Handle, handle, Shared, shared, CoreResult, CoreError};

// Re-export essential ECS types
pub use playground_core_ecs::{
    EntityId, ComponentId, ChannelId, ExecutionStage,
    EcsResult, EcsError,
};

// Re-export rendering types needed by plugins
pub use playground_core_rendering::{
    RenderCommand, RenderCommandBatch, Viewport,
    RenderError, RenderResult,
};

// Re-export UI types needed by plugins  
pub use playground_core_ui::{
    ElementId, ElementType, Style, Bounds,
    UiCommand, UiEvent, EventResult,
    LayoutType, FlexLayout, GridLayout,
};

// Re-export console types
pub use playground_core_console::{
    LogLevel, LogEntry, OutputStyle, Progress,
};

// Re-export client types
pub use playground_core_client::{
    ClientConfig, ClientState, ClientId, ClientStats, ClientCapabilities,
    RenderTarget, InputEvent, KeyCode, MouseButton,
};

// Re-export server types (for networking API)
pub use playground_core_server::{
    ConnectionInfo, ChannelInfo, MessagePriority,
};

/// Initialize the engine
/// 
/// This is the main entry point for apps. It initializes all engine systems
/// and sets up the command processors for cross-system communication.
pub async fn initialize_engine() -> CoreResult<()> {
    // This would:
    // 1. Create the unified World in systems/ecs
    // 2. Start all system command processors
    // 3. Register all systems with the World
    // 4. Set up the API gateway connections
    
    // For now, just succeed
    Ok(())
}

/// Shutdown the engine
/// 
/// Cleanly shuts down all systems and releases resources.
pub async fn shutdown_engine() -> CoreResult<()> {
    // This would:
    // 1. Stop all systems
    // 2. Clean up resources
    // 3. Shutdown command processors
    
    Ok(())
}

/// Update the engine
/// 
/// Called each frame to update all systems.
pub async fn update_engine(delta_time: f32) -> CoreResult<()> {
    // Forward to ECS World update
    ecs_api::update_world(delta_time).await
}

/// Get engine version information
pub fn engine_version() -> &'static str {
    "0.1.0"
}

/// Get engine capabilities
pub fn engine_capabilities() -> EngineCapabilities {
    EngineCapabilities {
        supports_networking: true,
        supports_rendering: true,
        supports_audio: false, // Not yet implemented
        supports_physics: false, // Not yet implemented
        max_entities: 1_000_000,
        max_components: 256,
        max_systems: 128,
    }
}

/// Engine capabilities information
#[derive(Debug, Clone)]
pub struct EngineCapabilities {
    pub supports_networking: bool,
    pub supports_rendering: bool,
    pub supports_audio: bool,
    pub supports_physics: bool,
    pub max_entities: usize,
    pub max_components: usize,
    pub max_systems: usize,
}