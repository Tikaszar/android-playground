//! Delete a snapshot

use playground_core_ecs::{World, StorageId, EcsResult, EcsError};

/// Delete a snapshot
pub async fn delete_snapshot(world: &World, snapshot_id: StorageId) -> EcsResult<()> {
    let mut storages = world.storages.write().await;
    if storages.remove(&snapshot_id).is_none() {
        return Err(EcsError::NotFound(format!("Snapshot {:?} not found", snapshot_id)));
    }

    Ok(())
}
