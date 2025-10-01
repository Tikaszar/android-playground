//! Get an entity by ID with current generation

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::EntityId;
use std::pin::Pin;
use std::future::Future;

/// Get an entity by ID (creates Entity handle with current generation)
pub fn get_entity(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
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

        // Return (EntityId, Generation) tuple
        #[derive(serde::Serialize)]
        struct GetEntityResult {
            id: EntityId,
            generation: playground_core_ecs::Generation,
        }

        let result_data = GetEntityResult {
            id: entity_id,
            generation,
        };

        // Serialize and return
        let result = bincode::serialize(&result_data)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
