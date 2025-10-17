//! Check if an entity has a component

use playground_core_ecs::{World, Entity, ComponentId, EcsResult};

/// Check if an entity has a component
pub async fn has_component(world: &World, entity: Entity, component_id: ComponentId) -> EcsResult<bool> {
    // Check if component exists
    let has_component = {
        let components = world.components.read().await;
        if let Some(entity_components) = components.get(&entity.id) {
            entity_components.contains_key(&component_id)
        } else {
            false
        }
    };

    Ok(has_component)
}
