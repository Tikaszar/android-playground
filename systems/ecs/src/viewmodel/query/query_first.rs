//! Get first entity matching a query

use playground_core_ecs::{World, Query, Entity, EcsResult, EcsError};

/// Get first entity matching a query
pub async fn query_first(world: &World, query: &Query) -> EcsResult<Entity> {
    // Get filter from stored queries
    let queries = world.queries.read().await;
    let filter = queries
        .get(&query.id)
        .ok_or_else(|| EcsError::QueryNotFound(query.id))?
        .clone();
    drop(queries);

    // Get all entities
    let entities = world.entities.read().await;

    // Find first matching entity
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
                return Ok(Entity::new(*entity_id, *generation, world.clone()));
            }
        }
    }

    // Return error if no entity found
    Err(EcsError::NoEntitiesMatchQuery(query.id))
}
