use playground_core_ecs::{self, EcsError};
use playground_core_types::Handle;

// Include the build-generated implementation
include!(concat!(env!("OUT_DIR"), "/system_loader_impl.rs"));

/// Load all available systems into core/ecs registry
/// This function's implementation is generated at build time from systems.toml
pub async fn load_all_systems() -> Result<(), EcsError> {
    load_all_systems_impl().await
}

/// Initialize all registered systems
pub async fn initialize_all_systems() -> Result<(), EcsError> {
    playground_core_ecs::initialize_all_registered().await
}

/// Get the render system if registered
pub async fn get_render_system() -> Option<Handle<playground_core_ecs::RenderSystem>> {
    playground_core_ecs::get_render_system().await
}

/// Get the network system if registered
pub async fn get_network_system() -> Option<Handle<playground_core_ecs::NetworkSystem>> {
    playground_core_ecs::get_network_system().await
}

/// Get the UI system if registered
pub async fn get_ui_system() -> Option<Handle<playground_core_ecs::UiSystem>> {
    playground_core_ecs::get_ui_system().await
}

/// Get the physics system if registered
pub async fn get_physics_system() -> Option<Handle<playground_core_ecs::PhysicsSystem>> {
    playground_core_ecs::get_physics_system().await
}