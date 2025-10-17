//! Spawn multiple entities in batch

use playground_core_ecs::{World, Entity, EntityId, Generation, Component, EcsResult};
use std::collections::HashMap;

/// Spawn multiple entities in batch
pub async fn spawn_batch(world: &World, batches: Vec<Vec<Component>>) -> EcsResult<Vec<Entity>> {
    let mut result_entities = Vec::new();

    // Spawn all entities
    for component_batch in batches {
        // Generate new entity ID
        let entity_id = EntityId(world.next_entity_id.fetch_add(1));
        let generation = Generation(1);

        // Store entity in World
        {
            let mut entities = world.entities.write().await;
            entities.insert(entity_id, generation);
        }

        // Store components
        if !component_batch.is_empty() {
            let mut components = world.components.write().await;
            let entity_components = components.entry(entity_id).or_insert_with(HashMap::new);
            for component in component_batch {
                entity_components.insert(component.component_id, component);
            }
        }

        result_entities.push(Entity { id: entity_id, generation });
    }

    Ok(result_entities)
}
