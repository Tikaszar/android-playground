//! Execute all systems in dependency order

use playground_core_ecs::{World, EcsResult};
use crate::viewmodel::system::schedule_systems::schedule_systems;
use crate::viewmodel::system::run_system::run_system;

/// Execute all systems in dependency order
pub async fn step_systems(world: &World, _delta_time: f32) -> EcsResult<()> {
    let scheduled = schedule_systems(world).await?;

    for system in scheduled {
        run_system(world, &system).await?;
    }

    Ok(())
}
