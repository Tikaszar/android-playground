//! Run multiple systems

use playground_core_ecs::{World, System, EcsResult};
use crate::viewmodel::system::run_system::run_system;

/// Run multiple systems
pub async fn run_systems(world: &World, systems: Vec<System>) -> EcsResult<()> {
    for system in systems {
        run_system(world, &system).await?;
    }
    Ok(())
}
