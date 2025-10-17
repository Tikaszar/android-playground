//! Get the event queue size

use playground_core_ecs::{World, EcsResult};

/// Get the event queue size
pub async fn get_event_queue_size(world: &World) -> EcsResult<usize> {
    // Get queue size
    let event_queue = world.event_queue.read().await;
    let size = event_queue.len();
    Ok(size)
}
