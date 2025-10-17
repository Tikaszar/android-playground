//! Get storage size in bytes

use playground_core_ecs::{Storage, EcsResult};

/// Get storage size in bytes
pub async fn get_storage_size(storage: &Storage) -> EcsResult<usize> {
    let size = storage.path.len() + storage.format.len();
    Ok(size)
}
