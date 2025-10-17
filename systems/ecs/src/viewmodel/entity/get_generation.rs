//! Get the current generation of an entity

use playground_core_ecs::{World, EntityId, Generation, EcsResult, EcsError};

/// Get the current generation of an entity
pub async fn get_generation(world: &World, entity_id: EntityId) -> EcsResult<Generation> {
    // Get entity generation
    let generation = {
        let entities = world.entities.read().await;
        entities.get(&entity_id)
            .copied()
            .ok_or(EcsError::EntityNotFound(entity_id))?
    };

    Ok(generation)
}
