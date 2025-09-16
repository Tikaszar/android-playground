//! Global registry for the World instance
//! 
//! This provides a singleton World that all systems can access.

use once_cell::sync::Lazy;
use playground_core_types::{Handle, Shared, shared};
use crate::{World, CoreError, CoreResult};

/// Global World instance
static WORLD: Lazy<Shared<Option<Handle<World>>>> = Lazy::new(|| shared(None));

/// Initialize the global World instance
/// 
/// This should be called once at application startup.
/// Returns an error if the world is already initialized.
pub async fn initialize_world() -> CoreResult<Handle<World>> {
    let mut guard = WORLD.write().await;
    if guard.is_some() {
        return Err(CoreError::AlreadyInitialized);
    }
    
    let world = World::new();
    *guard = Some(world.clone());
    Ok(world)
}

/// Get the global World instance
/// 
/// Returns an error if the world has not been initialized.
pub async fn get_world() -> CoreResult<Handle<World>> {
    let guard = WORLD.read().await;
    guard.as_ref()
        .cloned()
        .ok_or(CoreError::NotInitialized)
}

/// Check if the World has been initialized
pub async fn is_initialized() -> bool {
    let guard = WORLD.read().await;
    guard.is_some()
}

/// Shutdown and clear the World instance
/// 
/// This is useful for testing or clean shutdown.
pub async fn shutdown_world() -> CoreResult<()> {
    let mut guard = WORLD.write().await;
    if guard.is_none() {
        return Err(CoreError::NotInitialized);
    }
    
    *guard = None;
    Ok(())
}