//! Check if an entity has all specified components

use playground_core_ecs::{World, Entity, ComponentId, EcsResult};

/// Check if an entity has all specified components
pub async fn has_components(world: &World, entity: Entity, component_ids: Vec<ComponentId>) -> EcsResult<bool> {
    // Check all components
    let has_all = {
        let components = world.components.read().await;
        if let Some(entity_components) = components.get(&entity.id) {
            component_ids.iter().all(|id| entity_components.contains_key(id))
        } else {
            false
        }
    };

    Ok(has_all)
}
