//! Run a single system

use playground_core_ecs::{World, System, EcsResult};

/// Run a single system
pub async fn run_system(world: &World, system: &System) -> EcsResult<()> {
    // System execution logic would go here
    // For now, this is a minimal implementation that validates the system exists
    let systems = world.systems.read().await;
    if !systems.contains_key(&system.id) {
        return Err(playground_core_ecs::EcsError::SystemNotFound(format!("{:?}", system.id)));
    }

    // Actual system execution would involve:
    // 1. Execute the query associated with this system
    // 2. Process entities matching the query
    // 3. Update component data through component pools

    Ok(())
}
