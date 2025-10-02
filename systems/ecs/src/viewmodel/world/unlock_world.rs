//! Unlock the world

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::World;
use std::pin::Pin;
use std::future::Future;

/// Unlock the world
pub fn unlock_world(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize world
        let _world_arg: World = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let _world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // World unlocking would release the exclusive lock
        // For now, this is a placeholder

        Ok(Vec::new())
    })
}