//! Execute a query and return matching entities

use playground_core_ecs::{World, Query, Entity, EcsResult, EcsError};

/// Execute a query and return matching entities
pub async fn execute_query(world: &World, query: &Query) -> EcsResult<Vec<Entity>> {
    // Get filter from stored queries
    let queries = world.queries.read().await;
    let filter = queries
        .get(&query.id)
        .ok_or_else(|| EcsError::QueryNotFound(query.id))?
        .clone();
    drop(queries);

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

    Ok(matching_entities)
}
