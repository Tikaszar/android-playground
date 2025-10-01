//! Despawn an entity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Generation};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct DespawnEntityArgs {
    id: EntityId,
    generation: Generation,
}

/// Despawn an entity
pub fn despawn_entity(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize entity from args
        let entity_args: DespawnEntityArgs = bincode::deserialize(args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Remove entity from entities map
        let removed = {
            let mut entities = world.entities.write().await;
            entities.remove(&entity_args.id).is_some()
        };

        if !removed {
            return Err(ModuleError::Generic(format!("Entity {:?} not found", entity_args.id)));
        }

        // Remove all components for this entity
        {
            let mut components = world.components.write().await;
            components.remove(&entity_args.id);
        }

        Ok(Vec::new())
    })
}