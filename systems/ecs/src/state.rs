//! Global World state management

use once_cell::sync::OnceCell;
use playground_core_types::Handle;
use playground_core_ecs::World;

// THE single global World instance
static WORLD_INSTANCE: OnceCell<Handle<World>> = OnceCell::new();

/// Get the global World instance
pub fn get_world() -> Result<&'static Handle<World>, String> {
    WORLD_INSTANCE.get().ok_or_else(|| "World not initialized".to_string())
}

/// Initialize the global World instance
pub fn set_world(world: Handle<World>) -> Result<(), String> {
    WORLD_INSTANCE.set(world).map_err(|_| "World already initialized".to_string())
}

/// Check if World is initialized
pub fn is_initialized() -> bool {
    WORLD_INSTANCE.get().is_some()
}