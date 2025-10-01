//! Count components on an entity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::EntityId;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct CountComponentsArgs {
    entity_id: EntityId,
}

/// Count components on an entity
pub fn count_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: CountComponentsArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Count components
        let count = {
            let components = world.components.read().await;
            components.get(&args.entity_id).map(|c| c.len()).unwrap_or(0)
        };

        // Serialize and return
        let result = bincode::serialize(&count)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
