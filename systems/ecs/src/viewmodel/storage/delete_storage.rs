//! Delete storage

use playground_core_ecs::{World, Storage, EcsResult, EcsError};

/// Delete storage
pub async fn delete_storage(world: &World, storage: &Storage) -> EcsResult<()> {
    // Remove storage metadata from World
    let mut storages = world.storages.write().await;
    if storages.remove(&storage.id).is_none() {
        return Err(EcsError::NotFound(format!("Storage {:?} not found", storage.id)));
    }
    drop(storages);

    // In a complete implementation with actual file I/O, this would:
    // 1. Delete all files at storage.path
    // 2. Remove storage configuration from world.storages (done above)
    //
    // Since we don't have actual file system operations yet,
    // removing from world.storages is sufficient

    Ok(())
}
