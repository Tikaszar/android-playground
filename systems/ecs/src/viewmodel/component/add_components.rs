//! Add multiple components to an entity in batch

use playground_core_ecs::{World, Entity, Component, EcsResult, EcsError};
use std::collections::HashMap;

/// Add multiple components to an entity
pub async fn add_components(world: &World, entity: Entity, components: Vec<Component>) -> EcsResult<()> {
    // Check entity exists
    let exists = {
        let entities = world.entities.read().await;
        entities.contains_key(&entity.id)
    };

    if !exists {
        return Err(EcsError::EntityNotFound(entity.id));
    }

    // Add all components
    {
        let mut comps = world.components.write().await;
        let entity_components = comps.entry(entity.id).or_insert_with(HashMap::new);
        for component in components {
            entity_components.insert(component.component_id, component);
        }
    }

    Ok(())
}
