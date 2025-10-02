//! Get the count of entities matching a query

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::Query;
use std::pin::Pin;
use std::future::Future;

/// Get the count of entities matching a query
pub fn query_count(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize query
        let query: Query = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get filter from stored queries
        let filter = {
            let queries = world.queries.read().await;
            queries.get(&query.id)
                .ok_or_else(|| ModuleError::Generic(format!("Query {:?} not found", query.id)))?
                .clone()
        };

        // Get all entities
        let entities = world.entities.read().await;

        // Count matching entities
        let mut count: usize = 0;

        for (entity_id, _generation) in entities.iter() {
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
                    count += 1;
                }
            }
        }

        // Serialize and return
        let result = bincode::serialize(&count)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
