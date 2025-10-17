//! Get a system by ID

use playground_core_ecs::{World, System, SystemId, EcsResult};
use playground_modules_types::handle;

/// Get a system by ID
pub async fn get_system(world: &World, system_id: SystemId) -> EcsResult<System> {
    let systems = world.systems.read().await;

    if let Some((name, query, dependencies)) = systems.get(&system_id) {
        Ok(System::new(
            system_id,
            name.clone(),
            *query,
            dependencies.clone(),
            handle(world.clone())
        ))
    } else {
        Err(playground_core_ecs::EcsError::SystemNotFound(format!("{:?}", system_id)))
    }
}
