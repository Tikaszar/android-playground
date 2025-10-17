//! List all snapshots

use playground_core_ecs::{World, StorageId, EcsResult};

/// List all snapshots
pub async fn list_snapshots(world: &World) -> EcsResult<Vec<(StorageId, String)>> {
    let storages = world.storages.read().await;

    let snapshots: Vec<(StorageId, String)> = storages
        .iter()
        .filter(|(_, (_, format))| format == "snapshot")
        .map(|(id, (path, _))| {
            let name = path.strip_prefix("snapshots/").unwrap_or(path).to_string();
            (*id, name)
        })
        .collect();

    Ok(snapshots)
}
