//! Get all components for an entity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::EntityId;
use std::pin::Pin;
use std::future::Future;

/// Get all components for an entity
pub fn get_all_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize entity ID from args
        let entity_id: EntityId = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get all components
        let components_count = {
            let components = world.components.read().await;
            if let Some(entity_components) = components.get(&entity_id) {
                entity_components.len()
            } else {
                0
            }
        };

        // For now, just return the count as we can't easily serialize Component
        let result = bincode::serialize(&components_count)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}