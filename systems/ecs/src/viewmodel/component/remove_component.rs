//! Remove a component from an entity

use playground_core_ecs::{World, Entity, ComponentId, EcsResult, EcsError};

/// Remove a component from an entity
pub async fn remove_component(world: &World, entity: Entity, component_id: ComponentId) -> EcsResult<()> {
    // Remove component
    let removed = {
        let mut components = world.components.write().await;
        if let Some(entity_components) = components.get_mut(&entity.id) {
            entity_components.remove(&component_id).is_some()
        } else {
            false
        }
    };

    if !removed {
        return Err(EcsError::ComponentNotFound(component_id));
    }

    Ok(())
}
