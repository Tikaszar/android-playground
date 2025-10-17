//! Despawn multiple entities in batch

use playground_core_ecs::{World, Entity, EcsResult};

/// Despawn multiple entities in batch
pub async fn despawn_batch(world: &World, entities: Vec<Entity>) -> EcsResult<()> {
    // Despawn all entities
    {
        let mut world_entities = world.entities.write().await;
        let mut components = world.components.write().await;

        for entity in entities {
            world_entities.remove(&entity.id);
            components.remove(&entity.id);
        }
    }

    Ok(())
}
