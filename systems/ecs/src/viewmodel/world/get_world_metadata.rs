//! Get world metadata

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{World, WorldMetadata};
use std::pin::Pin;
use std::future::Future;

/// Get world metadata
pub fn get_world_metadata(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize world
        let _world_arg: World = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get current entity count
        let entity_count = {
            let entities = world.entities.read().await;
            entities.len()
        };

        // Create metadata
        let metadata = WorldMetadata {
            created_at: 0,  // Would track creation timestamp
            last_modified: 0,  // Would track modification timestamp
            version: 1,
            entity_count,
            is_locked: false,
        };

        // Serialize and return
        let result = bincode::serialize(&metadata)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}