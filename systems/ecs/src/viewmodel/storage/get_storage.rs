//! Get a storage by ID

use playground_core_ecs::{World, Storage, StorageId, EcsResult, EcsError};
use playground_modules_types::Handle;

/// Get a storage by ID
pub async fn get_storage(world: &World, storage_id: StorageId) -> EcsResult<Storage> {
    // Get storage metadata from World
    let storages = world.storages.read().await;
    let (path, format) = storages
        .get(&storage_id)
        .ok_or_else(|| EcsError::NotFound(format!("Storage {:?} not found", storage_id)))?
        .clone();
    drop(storages);

    // Recreate Handle<World> for Storage
    let world_handle: Handle<World> = unsafe {
        Handle::from_raw(world as *const World as *mut World)
    };

    // Create and return Storage
    Ok(Storage::new(storage_id, path, format, world_handle))
}
