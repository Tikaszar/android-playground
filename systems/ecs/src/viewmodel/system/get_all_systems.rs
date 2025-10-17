//! Get all registered systems

use playground_core_ecs::{World, System, EcsResult};
use playground_modules_types::handle;

/// Get all registered systems
pub async fn get_all_systems(world: &World) -> EcsResult<Vec<System>> {
    let systems = world.systems.read().await;

    let mut result = Vec::new();
    for (system_id, (name, query, dependencies)) in systems.iter() {
        result.push(System::new(
            *system_id,
            name.clone(),
            *query,
            dependencies.clone(),
            handle(world.clone())
        ));
    }

    Ok(result)
}
