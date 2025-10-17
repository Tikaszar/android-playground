//! Remove multiple components from an entity in batch

use playground_core_ecs::{World, Entity, ComponentId, EcsResult};

/// Remove multiple components from an entity
pub async fn remove_components(world: &World, entity: Entity, component_ids: Vec<ComponentId>) -> EcsResult<()> {
    // Remove all components
    {
        let mut components = world.components.write().await;
        if let Some(entity_components) = components.get_mut(&entity.id) {
            for component_id in component_ids {
                entity_components.remove(&component_id);
            }
        }
    }

    Ok(())
}
