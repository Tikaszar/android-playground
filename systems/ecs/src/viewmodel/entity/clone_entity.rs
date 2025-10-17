//! Clone an entity with all its components

use playground_core_ecs::{World, Entity, EntityId, Generation, EcsResult, EcsError};

/// Clone an entity with all its components
pub async fn clone_entity(world: &World, entity: Entity) -> EcsResult<Entity> {
    // Verify source entity exists
    let exists = {
        let entities = world.entities.read().await;
        entities.contains_key(&entity.id)
    };

    if !exists {
        return Err(EcsError::EntityNotFound(entity.id));
    }

    // Generate new entity ID
    let new_entity_id = EntityId(world.next_entity_id.fetch_add(1));
    let generation = Generation(1);

    // Store new entity
    {
        let mut entities = world.entities.write().await;
        entities.insert(new_entity_id, generation);
    }

    // Clone components
    {
        let components = world.components.read().await;
        if let Some(source_components) = components.get(&entity.id) {
            let cloned_components = source_components.clone();
            drop(components); // Release read lock

            let mut components = world.components.write().await;
            components.insert(new_entity_id, cloned_components);
        }
    }

    Ok(Entity { id: new_entity_id, generation })
}
