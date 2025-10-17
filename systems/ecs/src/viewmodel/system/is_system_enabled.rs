//! Check if a system is enabled

use playground_core_ecs::{World, SystemId, EcsResult};

/// Check if a system is enabled
pub async fn is_system_enabled(world: &World, system_id: SystemId) -> EcsResult<bool> {
    let systems = world.systems.read().await;
    Ok(systems.contains_key(&system_id))
}
