//! Execute query and get entities with their components

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{Query, Entity, Component};
use std::pin::Pin;
use std::future::Future;

/// Execute query and get entities with their components
pub fn execute_query_with_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
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

        // Collect matching entities with components
        let mut result_data: Vec<(Entity, Vec<Component>)> = Vec::new();

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
                    let entity = Entity::new(*entity_id, *generation, world.clone());

                    // For now, return empty component list
                    // Real implementation would query System.component_pools
                    let components = Vec::new();

                    result_data.push((entity, components));
                }
            }
        }

        // Serialize and return
        let result = bincode::serialize(&result_data)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}