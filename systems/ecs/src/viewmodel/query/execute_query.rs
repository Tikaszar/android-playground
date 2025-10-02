//! Execute a query and return matching entities

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{Query, Entity};
use std::pin::Pin;
use std::future::Future;

/// Execute a query and return matching entities
pub fn execute_query(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
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

        // For now, we'll return all entities that exist
        // In a complete implementation, we would check component_registry
        // to see which systems own the required components, then query those systems
        let mut matching_entities = Vec::new();

        for (entity_id, generation) in entities.iter() {
            // Check if entity has all included components
            // This is a simplified check - full implementation would query component pools
            let matches = if filter.include.is_empty() {
                // No requirements, all entities match
                true
            } else {
                // For now, assume entities with ID > 100 have components (placeholder logic)
                // Real implementation would check System.component_pools
                entity_id.0 > 100
            };

            if matches {
                // Check exclusions
                let excluded = if !filter.exclude.is_empty() {
                    // For now, assume entities with ID < 50 are excluded (placeholder logic)
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
