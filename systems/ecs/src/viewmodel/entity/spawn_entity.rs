//! Spawn a new entity with components

use playground_core_ecs::{World, Entity, EntityId, Generation, Component, EcsResult};
use std::collections::HashMap;

/// Spawn a new entity with components
pub async fn spawn_entity(world: &World, components: Vec<Component>) -> EcsResult<Entity> {
    // Generate new entity ID
    let entity_id = EntityId(world.next_entity_id.fetch_add(1));
    let generation = Generation(1);

    // Store entity in World
    {
        let mut entities = world.entities.write().await;
        entities.insert(entity_id, generation);
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
