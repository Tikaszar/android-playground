//! Despawn multiple entities in batch

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Generation};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct DespawnBatchArgs {
    entities: Vec<(EntityId, Generation)>,
}

/// Despawn multiple entities in batch
pub fn despawn_batch(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: DespawnBatchArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Despawn all entities
        {
            let mut entities = world.entities.write().await;
            let mut components = world.components.write().await;

            for (entity_id, _generation) in args.entities {
                entities.remove(&entity_id);
                components.remove(&entity_id);
            }
        }

        Ok(Vec::new())
    })
}
