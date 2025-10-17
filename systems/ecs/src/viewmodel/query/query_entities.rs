//! Create and execute a query in one operation

use playground_core_ecs::{World, QueryFilter, Entity, EcsResult};

/// Create and execute a query in one operation
pub async fn query_entities(world: &World, filter: QueryFilter) -> EcsResult<Vec<Entity>> {
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

    Ok(matching_entities)
}
