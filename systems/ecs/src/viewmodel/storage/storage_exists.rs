//! Check if storage exists

use playground_core_ecs::{Storage, EcsResult};

/// Check if storage exists
pub async fn storage_exists(storage: &Storage) -> EcsResult<bool> {
    // In a complete implementation with actual file I/O, this would:
    // 1. Check if files exist at storage.path
    // 2. Return true if storage files are present, false otherwise
    //
    // Since we don't have actual file system operations yet,
    // we consider a storage to exist if it has a valid ID
    // (it was created via create_storage)

    Ok(storage.id.0 > 0)
}
