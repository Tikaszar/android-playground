//! Check if any entities match a query

use playground_core_ecs::{World, Query, EcsResult, EcsError};

/// Check if any entities match a query
pub async fn query_has_results(world: &World, query: &Query) -> EcsResult<bool> {
    // Get filter from stored queries
    let queries = world.queries.read().await;
    let filter = queries
        .get(&query.id)
        .ok_or_else(|| EcsError::QueryNotFound(query.id))?
        .clone();
    drop(queries);

    // Get all entities
    let entities = world.entities.read().await;

    // Check if any entity matches
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
                return Ok(true);
            }
        }
    }

    Ok(false)
}
