//! Get system execution statistics

use playground_core_ecs::{World, SystemId, SystemStats, EcsResult};

/// Get system execution statistics
pub async fn get_system_stats(world: &World, system_id: SystemId) -> EcsResult<SystemStats> {
    let systems = world.systems.read().await;
    if !systems.contains_key(&system_id) {
        return Err(playground_core_ecs::EcsError::SystemNotFound(format!("{:?}", system_id)));
    }

    Ok(SystemStats::default())
}
