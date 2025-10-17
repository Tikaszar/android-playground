//! Execute query and get entities with their components

use playground_core_ecs::{World, Query, Entity, Component, EcsResult, EcsError};

/// Execute query and get entities with their components
pub async fn execute_query_with_components(world: &World, query: &Query) -> EcsResult<Vec<(Entity, Vec<Component>)>> {
    // Get filter from stored queries
    let queries = world.queries.read().await;
    let filter = queries
        .get(&query.id)
        .ok_or_else(|| EcsError::QueryNotFound(query.id))?
        .clone();
    drop(queries);

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

    Ok(result_data)
}
