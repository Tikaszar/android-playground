//! Get the current generation of an entity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::EntityId;
use std::pin::Pin;
use std::future::Future;

/// Get the current generation of an entity
pub fn get_generation(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize arguments
        let entity_id: EntityId = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get entity generation
        let generation = {
            let entities = world.entities.read().await;
            entities.get(&entity_id)
                .copied()
                .ok_or_else(|| ModuleError::Generic(format!("Entity {:?} not found", entity_id)))?
        };

        // Serialize and return
        let result = bincode::serialize(&generation)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
