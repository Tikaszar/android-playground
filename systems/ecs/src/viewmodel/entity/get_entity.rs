//! Get an entity by ID with current generation

use playground_core_ecs::{World, Entity, EntityId, EcsResult, EcsError};

/// Get an entity by ID (creates Entity handle with current generation)
pub async fn get_entity(world: &World, entity_id: EntityId) -> EcsResult<Entity> {
    // Get entity generation
    let generation = {
        let entities = world.entities.read().await;
        entities.get(&entity_id)
            .copied()
            .ok_or(EcsError::EntityNotFound(entity_id))?
    };

    Ok(Entity { id: entity_id, generation })
}
