//! Initialize the World instance

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

/// Initialize the world
pub fn initialize_world(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let _args = args.to_vec();
    Box::pin(async move {
        // Check if already initialized
        if crate::state::is_initialized() {
            return Err(ModuleError::Generic("World already initialized".to_string()));
        }

        // Create new World instance
        let world = playground_core_ecs::World::new();

        // Store in global state
        crate::state::set_world(world.clone())
            .map_err(|e| ModuleError::Generic(e))?;

        // Return empty success (world is now accessible via get_world)
        Ok(Vec::new())
    })
}