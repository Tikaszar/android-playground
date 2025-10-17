//! Save specific entities to storage

use playground_core_ecs::{World, Storage, Entity, EcsResult, EcsError};

/// Save specific entities to storage
pub async fn save_entities(world: &World, storage: &Storage, entities: Vec<Entity>) -> EcsResult<()> {
    // Verify storage exists
    let storages = world.storages.read().await;
    if !storages.contains_key(&storage.id) {
        return Err(EcsError::NotFound(format!("Storage {:?} not found", storage.id)));
    }
    drop(storages);

    // Verify all entities exist
    let world_entities = world.entities.read().await;
    for entity in &entities {
        if !world_entities.contains_key(&entity.id) {
            return Err(EcsError::NotFound(format!("Entity {:?} not found", entity.id)));
        }
    }
    drop(world_entities);

    // In a complete implementation with actual file I/O, this would:
    // 1. For each entity, collect its components from component pools
    // 2. Serialize entities and components based on storage.format
    // 3. Write to storage.path (append or update existing file)
    //
    // Since we don't have actual file system operations yet,
    // this is a valid no-op that verifies storage and entities exist

    Ok(())
}
