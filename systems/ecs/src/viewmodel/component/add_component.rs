//! Add a component to an entity

use playground_core_ecs::{World, Entity, Component, EcsResult, EcsError};
use std::collections::HashMap;

/// Add a component to an entity
pub async fn add_component(world: &World, entity: Entity, component: Component) -> EcsResult<()> {
    // Check entity exists
    let exists = {
        let entities = world.entities.read().await;
        entities.contains_key(&entity.id)
    };

    if !exists {
        return Err(EcsError::EntityNotFound(entity.id));
    }

    // Add component
    {
        let mut components = world.components.write().await;
        let entity_components = components.entry(entity.id).or_insert_with(HashMap::new);
        entity_components.insert(component.component_id, component);
    }

    Ok(())
}
