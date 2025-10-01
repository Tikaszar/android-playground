//! Check if an entity has all specified components

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, ComponentId};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct HasComponentsArgs {
    entity_id: EntityId,
    component_ids: Vec<ComponentId>,
}

/// Check if an entity has all specified components
pub fn has_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: HasComponentsArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Check all components
        let has_all = {
            let components = world.components.read().await;
            if let Some(entity_components) = components.get(&args.entity_id) {
                args.component_ids.iter().all(|id| entity_components.contains_key(id))
            } else {
                false
            }
        };

        // Serialize and return
        let result = bincode::serialize(&has_all)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
