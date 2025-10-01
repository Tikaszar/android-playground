//! Check if an entity is alive (valid generation)

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Generation};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct IsAliveArgs {
    id: EntityId,
    generation: Generation,
}

/// Check if an entity is alive (valid generation)
pub fn is_alive(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize entity from args
        let entity_args: IsAliveArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Check if entity is alive (generation matches)
        let is_alive = {
            let entities = world.entities.read().await;
            match entities.get(&entity_args.id) {
                Some(generation) => *generation == entity_args.generation,
                None => false,
            }
        };

        // Serialize result
        let result = bincode::serialize(&is_alive)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}