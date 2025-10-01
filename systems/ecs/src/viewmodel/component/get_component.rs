//! Get a component from an entity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, ComponentId};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct GetComponentArgs {
    entity_id: EntityId,
    component_id: ComponentId,
}

/// Get a component from an entity
pub fn get_component(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize args
        let args: GetComponentArgs = bincode::deserialize(args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get component
        let component = {
            let components = world.components.read().await;
            if let Some(entity_components) = components.get(&args.entity_id) {
                entity_components.get(&args.component_id).cloned()
            } else {
                None
            }
        };

        match component {
            Some(comp) => {
                // Component contains Bytes which doesn't serialize directly
                // Return the raw component data
                Ok(comp.data.to_vec())
            }
            None => Err(ModuleError::Generic(format!(
                "Component {:?} not found on entity {:?}",
                args.component_id, args.entity_id
            ))),
        }
    })
}