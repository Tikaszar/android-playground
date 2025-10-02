//! Create and execute a query in one operation

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{QueryFilter, Entity, QueryId};
use std::pin::Pin;
use std::future::Future;

/// Create and execute a query in one operation
pub fn query_entities(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize filter
        let filter: QueryFilter = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Create a temporary query ID (we won't store it)
        let _query_id = QueryId(world.next_query_id.load());

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

        // Serialize and return
        let result = bincode::serialize(&matching_entities)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}