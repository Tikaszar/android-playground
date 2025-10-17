//! Enable a system

use playground_core_ecs::{World, SystemId, EcsResult};

/// Enable a system
pub async fn enable_system(world: &World, system_id: SystemId) -> EcsResult<()> {
    let systems = world.systems.read().await;
    if !systems.contains_key(&system_id) {
        return Err(playground_core_ecs::EcsError::SystemNotFound(format!("{:?}", system_id)));
    }
    Ok(())
}
