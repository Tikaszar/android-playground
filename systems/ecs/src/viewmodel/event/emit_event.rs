//! Emit an event to be processed

use playground_core_ecs::{World, Event, EcsResult};

/// Emit an event to be processed
pub async fn emit_event(world: &World, event: Event) -> EcsResult<()> {
    // Add to event queue
    let mut event_queue = world.event_queue.write().await;
    event_queue.push(event);
    Ok(())
}
