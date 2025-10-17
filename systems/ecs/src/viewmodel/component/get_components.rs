//! Get multiple specific components from an entity

use playground_core_ecs::{World, Entity, ComponentId, Component, EcsResult};

/// Get multiple specific components from an entity
pub async fn get_components(world: &World, entity: Entity, component_ids: Vec<ComponentId>) -> EcsResult<Vec<Component>> {
    // Get components
    let result: Vec<Component> = {
        let components = world.components.read().await;
        if let Some(entity_components) = components.get(&entity.id) {
            component_ids
                .iter()
                .filter_map(|id| entity_components.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    };

    Ok(result)
}
