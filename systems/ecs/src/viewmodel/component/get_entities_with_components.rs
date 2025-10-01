//! Get all entities that have all specified components

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{ComponentId, EntityId, Generation};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct GetEntitiesWithComponentsArgs {
    component_ids: Vec<ComponentId>,
}

/// Get all entities that have all specified components
pub fn get_entities_with_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: GetEntitiesWithComponentsArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Find all entities with all components - return as (EntityId, Generation) tuples
        let entities: Vec<(EntityId, Generation)> = {
            let components = world.components.read().await;
            let entity_gens = world.entities.read().await;

            components
                .iter()
                .filter(|(_, entity_components)| {
                    args.component_ids.iter().all(|id| entity_components.contains_key(id))
                })
                .filter_map(|(entity_id, _)| {
                    entity_gens.get(entity_id).map(|generation| (*entity_id, *generation))
                })
                .collect()
        };

        // Serialize and return
        let result = bincode::serialize(&entities)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
