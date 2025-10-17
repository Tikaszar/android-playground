//! Get all storages

use playground_core_ecs::{World, Storage, EcsResult};
use playground_modules_types::Handle;

/// Get all storages
pub async fn get_all_storages(world: &World) -> EcsResult<Vec<Storage>> {
    // Get all storage metadata from World
    let storages = world.storages.read().await;

    // Recreate Handle<World> for Storage instances
    let world_handle: Handle<World> = unsafe {
        Handle::from_raw(world as *const World as *mut World)
    };

    // Convert to Storage instances
    let result: Vec<Storage> = storages
        .iter()
        .map(|(storage_id, (path, format))| {
            Storage::new(*storage_id, path.clone(), format.clone(), world_handle.clone())
        })
        .collect();

    Ok(result)
}
