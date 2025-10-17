//! Clear all components from an entity

use playground_core_ecs::{World, Entity, EcsResult};

/// Clear all components from an entity
pub async fn clear_components(world: &World, entity: Entity) -> EcsResult<()> {
    // Clear all components
    {
        let mut components = world.components.write().await;
        components.remove(&entity.id);
    }

    Ok(())
}
