//! Clone the world

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::World;
use playground_core_types::Handle;
use std::pin::Pin;
use std::future::Future;

/// Clone the world
pub fn clone_world(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize world
        let _world_arg: World = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get current World
        let _world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Create new world
        let new_world = World::new();

        // Cloning would typically:
        // 1. Copy all entities
        // 2. Copy all components
        // 3. Copy all systems
        // 4. Copy all queries
        // For now, we just create a fresh world

        // Serialize the handle
        let result = bincode::serialize(&new_world)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}