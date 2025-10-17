//! Register a new system

use playground_core_ecs::{World, System, SystemId, QueryId, EcsResult};
use playground_modules_types::handle;

/// Register a new system
pub async fn register_system(
    world: &World,
    name: String,
    query: QueryId,
    dependencies: Vec<SystemId>
) -> EcsResult<System> {
    // Generate new system ID
    let system_id = SystemId(world.next_system_id.fetch_add(1) as u32);

    // Store system metadata in World
    {
        let mut systems = world.systems.write().await;
        systems.insert(system_id, (name.clone(), query, dependencies.clone()));
    }

    // Create System handle
    let system = System::new(system_id, name, query, dependencies, handle(world.clone()));

    Ok(system)
}
