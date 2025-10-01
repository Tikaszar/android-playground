//! Get multiple specific components from an entity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, ComponentId, Component};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct GetComponentsArgs {
    entity_id: EntityId,
    component_ids: Vec<ComponentId>,
}

/// Get multiple specific components from an entity
pub fn get_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: GetComponentsArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get components
        let result: Vec<Component> = {
            let components = world.components.read().await;
            if let Some(entity_components) = components.get(&args.entity_id) {
                args.component_ids
                    .iter()
                    .filter_map(|id| entity_components.get(id).cloned())
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Serialize and return
        let result = bincode::serialize(&result)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
