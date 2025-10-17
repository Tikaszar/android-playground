//! Clear storage contents

use playground_core_ecs::{Storage, EcsResult};

/// Clear storage contents
pub async fn clear_storage(storage: &Storage) -> EcsResult<()> {
    // In a complete implementation with actual file I/O, this would:
    // 1. Delete all data files at storage.path
    // 2. Keep the storage configuration in world.storages
    // 3. Empty the storage but keep it registered
    //
    // Since we don't have actual file system operations yet,
    // this is a valid no-op

    let _ = storage;
    Ok(())
}
