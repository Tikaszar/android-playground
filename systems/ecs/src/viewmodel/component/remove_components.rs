//! Remove multiple components from an entity in batch

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, ComponentId};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct RemoveComponentsArgs {
    entity_id: EntityId,
    component_ids: Vec<ComponentId>,
}

/// Remove multiple components from an entity
pub fn remove_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: RemoveComponentsArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Remove all components
        {
            let mut components = world.components.write().await;
            if let Some(entity_components) = components.get_mut(&args.entity_id) {
                for component_id in args.component_ids {
                    entity_components.remove(&component_id);
                }
            }
        }

        Ok(Vec::new())
    })
}
