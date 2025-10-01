//! Get all entities in the world

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Generation};
use std::pin::Pin;
use std::future::Future;

/// Get all entities in the world
pub fn get_all_entities(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // No arguments needed for this function

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get all entities
        let entities = {
            let entities_map = world.entities.read().await;
            entities_map.iter()
                .map(|(id, generation)| (*id, *generation))
                .collect::<Vec<(EntityId, Generation)>>()
        };

        // Serialize and return
        let result = bincode::serialize(&entities)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
