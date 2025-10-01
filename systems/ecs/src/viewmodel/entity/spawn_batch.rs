//! Spawn multiple entities in batch

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Generation, Component};
use std::pin::Pin;
use std::future::Future;
use std::sync::atomic::Ordering;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct SpawnBatchArgs {
    batches: Vec<Vec<Component>>,
}

#[derive(serde::Serialize)]
struct SpawnBatchResult {
    entities: Vec<(EntityId, Generation)>,
}

/// Spawn multiple entities in batch
pub fn spawn_batch(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: SpawnBatchArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        let mut result_entities = Vec::new();

        // Spawn all entities
        for component_batch in args.batches {
            // Generate new entity ID
            let entity_id = EntityId(world.next_entity_id.fetch_add(1, Ordering::SeqCst));
            let generation = Generation(1);

            // Store entity in World
            {
                let mut entities = world.entities.write().await;
                entities.insert(entity_id, generation);
            }

            // Store components
            if !component_batch.is_empty() {
                let mut components = world.components.write().await;
                let entity_components = components.entry(entity_id).or_insert_with(HashMap::new);
                for component in component_batch {
                    entity_components.insert(component.component_id, component);
                }
            }

            result_entities.push((entity_id, generation));
        }

        // Serialize and return
        let result_data = SpawnBatchResult {
            entities: result_entities,
        };

        let result = bincode::serialize(&result_data)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
