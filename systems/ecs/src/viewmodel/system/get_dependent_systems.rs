//! Get systems that depend on a given system

use playground_core_ecs::{World, SystemId, EcsResult};

/// Get systems that depend on a given system
pub async fn get_dependent_systems(world: &World, system_id: SystemId) -> EcsResult<Vec<SystemId>> {
    let systems = world.systems.read().await;

    let mut dependents = Vec::new();
    for (id, (_, _, dependencies)) in systems.iter() {
        if dependencies.contains(&system_id) {
            dependents.push(*id);
        }
    }

    Ok(dependents)
}
