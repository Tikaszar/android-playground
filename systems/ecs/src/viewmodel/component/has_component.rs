//! Check if an entity has a component

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, ComponentId};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct HasComponentArgs {
    entity_id: EntityId,
    component_id: ComponentId,
}

/// Check if an entity has a component
pub fn has_component(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize args
        let args: HasComponentArgs = bincode::deserialize(args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Check if component exists
        let has_component = {
            let components = world.components.read().await;
            if let Some(entity_components) = components.get(&args.entity_id) {
                entity_components.contains_key(&args.component_id)
            } else {
                false
            }
        };

        // Serialize result
        let result = bincode::serialize(&has_component)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}