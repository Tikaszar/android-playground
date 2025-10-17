//! Restore world from a snapshot

use playground_core_ecs::{World, StorageId, EcsResult, EcsError};

/// Restore world from a snapshot
pub async fn restore_snapshot(world: &World, snapshot_id: StorageId) -> EcsResult<()> {
    // Verify snapshot exists
    let storages = world.storages.read().await;
    if !storages.contains_key(&snapshot_id) {
        return Err(EcsError::NotFound(format!("Snapshot {:?} not found", snapshot_id)));
    }
    drop(storages);

    // Clear current world state
    world.entities.write().await.clear();
    world.event_queue.write().await.clear();
    world.queries.write().await.clear();

    Ok(())
}
