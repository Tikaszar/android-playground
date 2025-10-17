//! Update system dependencies

use playground_core_ecs::{World, SystemId, EcsResult};

/// Update system dependencies
pub async fn update_system_dependencies(
    world: &World,
    system_id: SystemId,
    dependencies: Vec<SystemId>
) -> EcsResult<()> {
    let mut systems = world.systems.write().await;

    if let Some((name, query, _)) = systems.get(&system_id) {
        let name = name.clone();
        let query = *query;
        systems.insert(system_id, (name, query, dependencies));
        Ok(())
    } else {
        Err(playground_core_ecs::EcsError::SystemNotFound(format!("{:?}", system_id)))
    }
}
