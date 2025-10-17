//! Get system dependencies

use playground_core_ecs::{World, SystemId, EcsResult};

/// Get system dependencies
pub async fn get_system_dependencies(world: &World, system_id: SystemId) -> EcsResult<Vec<SystemId>> {
    let systems = world.systems.read().await;

    if let Some((_, _, dependencies)) = systems.get(&system_id) {
        Ok(dependencies.clone())
    } else {
        Err(playground_core_ecs::EcsError::SystemNotFound(format!("{:?}", system_id)))
    }
}
