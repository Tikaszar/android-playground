//! Check if an entity is alive (valid generation)

use playground_core_ecs::{World, Entity, EcsResult};

/// Check if an entity is alive (valid generation)
pub async fn is_alive(world: &World, entity: Entity) -> EcsResult<bool> {
    // Check if entity is alive (generation matches)
    let is_alive = {
        let entities = world.entities.read().await;
        match entities.get(&entity.id) {
            Some(generation) => *generation == entity.generation,
            None => false,
        }
    };

    Ok(is_alive)
}
