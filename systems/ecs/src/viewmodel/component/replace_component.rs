//! Replace a component on an entity (add or update)

use playground_core_ecs::{World, Entity, Component, EcsResult, EcsError};
use std::collections::HashMap;

/// Replace a component on an entity (add or update)
pub async fn replace_component(world: &World, entity: Entity, component: Component) -> EcsResult<()> {
    // Check entity exists
    let exists = {
        let entities = world.entities.read().await;
        entities.contains_key(&entity.id)
    };

    if !exists {
        return Err(EcsError::EntityNotFound(entity.id));
    }

    // Replace component (insert overwrites existing)
    {
        let mut components = world.components.write().await;
        let entity_components = components.entry(entity.id).or_insert_with(HashMap::new);
        entity_components.insert(component.component_id, component);
    }

    Ok(())
}
