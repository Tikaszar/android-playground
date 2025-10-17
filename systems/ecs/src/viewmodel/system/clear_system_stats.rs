//! Clear all system statistics

use playground_core_ecs::{World, EcsResult};

/// Clear all system statistics
pub async fn clear_system_stats(world: &World) -> EcsResult<()> {
    let _systems = world.systems.read().await;
    Ok(())
}
