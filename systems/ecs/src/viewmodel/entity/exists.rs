//! Check if an entity exists

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::EntityId;
use std::pin::Pin;
use std::future::Future;

/// Check if an entity exists
pub fn exists(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize entity ID from args
        let entity_id: EntityId = bincode::deserialize(args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Check if entity exists
        let exists = {
            let entities = world.entities.read().await;
            entities.contains_key(&entity_id)
        };

        // Serialize result
        let result = bincode::serialize(&exists)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}