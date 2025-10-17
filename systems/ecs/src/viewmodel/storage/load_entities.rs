//! Load entities from storage

use playground_core_ecs::{World, Storage, Entity, EcsResult, EcsError};

/// Load entities from storage
pub async fn load_entities(world: &World, storage: &Storage) -> EcsResult<Vec<Entity>> {
    // Verify storage exists
    let storages = world.storages.read().await;
    if !storages.contains_key(&storage.id) {
        return Err(EcsError::NotFound(format!("Storage {:?} not found", storage.id)));
    }
    drop(storages);

    // In a complete implementation with actual file I/O, this would:
    // 1. Read serialized entity data from storage.path
    // 2. Deserialize based on storage.format (JSON, Binary, etc.)
    // 3. Create entities in world.entities with new IDs
    // 4. Create components in component pools
    // 5. Return the loaded entities
    //
    // Since we don't have actual file system operations yet,
    // return empty vector (no entities loaded)

    Ok(Vec::new())
}
