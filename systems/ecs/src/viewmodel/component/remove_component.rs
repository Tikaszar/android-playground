//! Remove a component from an entity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, ComponentId};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct RemoveComponentArgs {
    entity_id: EntityId,
    component_id: ComponentId,
}

/// Remove a component from an entity
pub fn remove_component(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize args
        let args: RemoveComponentArgs = bincode::deserialize(args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Remove component
        let removed = {
            let mut components = world.components.write().await;
            if let Some(entity_components) = components.get_mut(&args.entity_id) {
                entity_components.remove(&args.component_id).is_some()
            } else {
                false
            }
        };

        if !removed {
            return Err(ModuleError::Generic(format!(
                "Component {:?} not found on entity {:?}",
                args.component_id, args.entity_id
            )));
        }

        Ok(Vec::new())
    })
}