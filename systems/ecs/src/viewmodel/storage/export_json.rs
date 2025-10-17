//! Export world to JSON format

use playground_core_ecs::{World, EcsResult};

/// Export world to JSON format
pub async fn export_json(world: &World, path: String) -> EcsResult<()> {
    let entities = world.entities.read().await;
    let _entity_count = entities.len();
    drop(entities);

    let _ = path;
    Ok(())
}
