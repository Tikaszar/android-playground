//! Clear the event queue without processing

use playground_core_ecs::{World, EcsResult};

/// Clear the event queue without processing
pub async fn clear_event_queue(world: &World) -> EcsResult<()> {
    // Clear event queue
    let mut event_queue = world.event_queue.write().await;
    event_queue.clear();
    Ok(())
}
