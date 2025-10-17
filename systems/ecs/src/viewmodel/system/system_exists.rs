//! Check if system exists

use playground_core_ecs::{World, SystemId, EcsResult};

/// Check if system exists
pub async fn system_exists(world: &World, system_id: SystemId) -> EcsResult<bool> {
    let systems = world.systems.read().await;
    Ok(systems.contains_key(&system_id))
}
