//! Unregister a system

use playground_core_ecs::{World, System, EcsResult};

/// Unregister a system
pub async fn unregister_system(world: &World, system: &System) -> EcsResult<()> {
    let system_id = system.id;

    // Remove system metadata from World
    {
        let mut systems = world.systems.write().await;
        systems.remove(&system_id);
    }

    Ok(())
}
