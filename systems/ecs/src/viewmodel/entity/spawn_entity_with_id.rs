//! Spawn entity with specific ID (useful for deserialization)

use playground_core_ecs::{World, Entity, EntityId, Generation, Component, EcsResult, EcsError};
use std::collections::HashMap;

/// Spawn entity with specific ID (useful for deserialization)
pub async fn spawn_entity_with_id(world: &World, entity_id: EntityId, components: Vec<Component>) -> EcsResult<Entity> {
    let generation = Generation(1);

    // Check if entity already exists
    {
        let entities = world.entities.read().await;
        if entities.contains_key(&entity_id) {
            return Err(EcsError::EntityAlreadyExists(entity_id));
        }
    }

    // Store entity in World
    {
        let mut entities = world.entities.write().await;
        entities.insert(entity_id, generation);
    }

    // Update next_entity_id if needed
    {
        let current_next = world.next_entity_id.load();
        if entity_id.0 >= current_next {
            world.next_entity_id.store(entity_id.0 + 1);
        }
    }

    // Store components
    if !components.is_empty() {
        let mut comps = world.components.write().await;
        let entity_components = comps.entry(entity_id).or_insert_with(HashMap::new);
        for component in components {
            entity_components.insert(component.component_id, component);
        }
    }

    Ok(Entity { id: entity_id, generation })
}
