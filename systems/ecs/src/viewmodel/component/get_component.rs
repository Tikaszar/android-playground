//! Get a component from an entity

use playground_core_ecs::{World, Entity, ComponentId, Component, EcsResult, EcsError};

/// Get a component from an entity
pub async fn get_component(world: &World, entity: Entity, component_id: ComponentId) -> EcsResult<Component> {
    // Get component
    let component = {
        let components = world.components.read().await;
        if let Some(entity_components) = components.get(&entity.id) {
            entity_components.get(&component_id).cloned()
        } else {
            None
        }
    };

    component.ok_or(EcsError::ComponentNotFound(component_id))
}
