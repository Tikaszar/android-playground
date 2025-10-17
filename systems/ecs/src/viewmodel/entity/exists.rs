//! Check if an entity exists

use playground_core_ecs::{World, Entity, EcsResult};

/// Check if an entity exists
pub async fn exists(world: &World, entity: Entity) -> EcsResult<bool> {
    // Check if entity exists
    let exists = {
        let entities = world.entities.read().await;
        entities.contains_key(&entity.id)
    };

    Ok(exists)
}
