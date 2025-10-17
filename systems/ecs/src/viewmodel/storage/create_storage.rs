//! Create a new storage configuration

use playground_core_ecs::{World, Storage, StorageId, EcsResult};
use playground_modules_types::Handle;

/// Create a new storage configuration
pub async fn create_storage(world: &World, path: String, format: String) -> EcsResult<Storage> {
    // Generate new storage ID
    let storage_id = StorageId(world.next_storage_id.fetch_add(1));

    // Store metadata in World
    {
        let mut storages = world.storages.write().await;
        storages.insert(storage_id, (path.clone(), format.clone()));
    }

    // Create Storage model - includes Handle<World> so Storage can access World
    let world_handle: Handle<World> = unsafe {
        // SAFETY: We know world is backed by an Arc from Handle<World>
        // We need to recreate the Handle to pass to Storage::new
        Handle::from_raw(world as *const World as *mut World)
    };

    let storage = Storage::new(storage_id, path, format, world_handle);

    Ok(storage)
}
