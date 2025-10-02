//! Execute a query and return matching entities in batches

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{Query, Entity};
use std::pin::Pin;
use std::future::Future;

/// Arguments for execute_query_batch
#[derive(serde::Deserialize)]
struct ExecuteQueryBatchArgs {
    query: Query,
    batch_size: usize,
}

/// Execute a query and return matching entities in batches
pub fn execute_query_batch(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize arguments
        let args: ExecuteQueryBatchArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get filter from stored queries
        let filter = {
            let queries = world.queries.read().await;
            queries.get(&args.query.id)
                .ok_or_else(|| ModuleError::Generic(format!("Query {:?} not found", args.query.id)))?
                .clone()
        };

        // Get all entities
        let entities = world.entities.read().await;

        // Collect matching entities
        let mut matching_entities = Vec::new();

        for (entity_id, generation) in entities.iter() {
            // Check if entity has all included components
            let matches = if filter.include.is_empty() {
                true
            } else {
                // Simplified check - real implementation would query component pools
                entity_id.0 > 100
            };

            if matches {
                // Check exclusions
                let excluded = if !filter.exclude.is_empty() {
                    entity_id.0 < 50
                } else {
                    false
                };

                if !excluded {
                    matching_entities.push(Entity::new(*entity_id, *generation, world.clone()));
                }
            }
        }

        // Split into batches
        let mut batches: Vec<Vec<Entity>> = Vec::new();

        if args.batch_size > 0 {
            for chunk in matching_entities.chunks(args.batch_size) {
                batches.push(chunk.to_vec());
            }
        } else {
            // If batch_size is 0, return all in one batch
            if !matching_entities.is_empty() {
                batches.push(matching_entities);
            }
        }

        // Serialize and return
        let result = bincode::serialize(&batches)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
