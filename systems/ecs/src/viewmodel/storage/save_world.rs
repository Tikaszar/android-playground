//! Save the entire world to storage

use playground_core_ecs::{World, Storage, EcsResult, EcsError};

/// Save the entire world to storage
pub async fn save_world(world: &World, storage: &Storage) -> EcsResult<()> {
    // Verify storage exists
    let storages = world.storages.read().await;
    if !storages.contains_key(&storage.id) {
        return Err(EcsError::NotFound(format!("Storage {:?} not found", storage.id)));
    }
    drop(storages);

    // In a complete implementation with actual file I/O, this would:
    // 1. Collect all entities from world.entities
    // 2. For each entity, collect its components from component pools
    // 3. Serialize everything based on storage.format (JSON, Binary, etc.)
    // 4. Write to storage.path
    //
    // Since we don't have actual file system operations yet,
    // this is a valid no-op that verifies storage exists

    Ok(())
}
