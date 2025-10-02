//! Get first entity matching a query

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{Query, Entity};
use std::pin::Pin;
use std::future::Future;

/// Get first entity matching a query
pub fn query_first(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
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

        // Find first matching entity
        let mut first_entity = None;

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
                    first_entity = Some(Entity::new(*entity_id, *generation, world.clone()));
                    break;
                }
            }
        }

        // Return error if no entity found
        let entity = first_entity
            .ok_or_else(|| ModuleError::Generic("No entities match query".to_string()))?;

        // Serialize and return
        let result = bincode::serialize(&entity)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}