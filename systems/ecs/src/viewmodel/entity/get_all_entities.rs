//! Get all entities in the world

use playground_core_ecs::{World, Entity, EcsResult};

/// Get all entities in the world
pub async fn get_all_entities(world: &World) -> EcsResult<Vec<Entity>> {
    // Get all entities
    let entities = {
        let entities_map = world.entities.read().await;
        entities_map.iter()
            .map(|(id, generation)| Entity { id: *id, generation: *generation })
            .collect::<Vec<Entity>>()
    };

    Ok(entities)
}
