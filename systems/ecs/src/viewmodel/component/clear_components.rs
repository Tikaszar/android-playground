//! Clear all components from an entity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::EntityId;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct ClearComponentsArgs {
    entity_id: EntityId,
}

/// Clear all components from an entity
pub fn clear_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: ClearComponentsArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Clear all components
        {
            let mut components = world.components.write().await;
            components.remove(&args.entity_id);
        }

        Ok(Vec::new())
    })
}
