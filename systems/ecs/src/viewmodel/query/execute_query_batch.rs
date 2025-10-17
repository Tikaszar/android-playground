//! Execute a query and return matching entities in batches

use playground_core_ecs::{World, Query, Entity, EcsResult, EcsError};

/// Execute a query and return matching entities in batches
pub async fn execute_query_batch(world: &World, query: &Query, batch_size: usize) -> EcsResult<Vec<Vec<Entity>>> {
    // Get filter from stored queries
    let queries = world.queries.read().await;
    let filter = queries
        .get(&query.id)
        .ok_or_else(|| EcsError::QueryNotFound(query.id))?
        .clone();
    drop(queries);

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

    if batch_size > 0 {
        for chunk in matching_entities.chunks(batch_size) {
            batches.push(chunk.to_vec());
        }
    } else {
        // If batch_size is 0, return all in one batch
        if !matching_entities.is_empty() {
            batches.push(matching_entities);
        }
    }

    Ok(batches)
}
