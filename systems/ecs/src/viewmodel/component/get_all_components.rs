//! Get all components for an entity

use playground_core_ecs::{World, Entity, Component, EcsResult};

/// Get all components for an entity
pub async fn get_all_components(world: &World, entity: Entity) -> EcsResult<Vec<Component>> {
    // Get all components
    let components_vec = {
        let components = world.components.read().await;
        if let Some(entity_components) = components.get(&entity.id) {
            entity_components.values().cloned().collect()
        } else {
            Vec::new()
        }
    };

    Ok(components_vec)
}
