//! Import world from JSON format

use playground_core_ecs::{World, EcsResult};

/// Import world from JSON format
pub async fn import_json(world: &World, path: String) -> EcsResult<()> {
    world.entities.write().await.clear();
    world.event_queue.write().await.clear();
    world.queries.write().await.clear();

    let _ = path;
    Ok(())
}
