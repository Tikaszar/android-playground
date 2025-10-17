//! Despawn an entity

use playground_core_ecs::{World, Entity, EcsResult, EcsError};

/// Despawn an entity
pub async fn despawn_entity(world: &World, entity: Entity) -> EcsResult<()> {
    // Remove entity from entities map
    let removed = {
        let mut entities = world.entities.write().await;
        entities.remove(&entity.id).is_some()
    };

    if !removed {
        return Err(EcsError::EntityNotFound(entity.id));
    }

    // Remove all components for this entity
    {
        let mut components = world.components.write().await;
        components.remove(&entity.id);
    }

    Ok(())
}
