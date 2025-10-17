//! Emit an event with specific priority

use playground_core_ecs::{World, Event, Priority, EcsResult};

/// Emit an event with specific priority
pub async fn emit_event_with_priority(world: &World, event: Event, priority: Priority) -> EcsResult<()> {
    // Create event with priority
    let event = event.with_priority(priority);

    // Add to event queue
    let mut event_queue = world.event_queue.write().await;
    event_queue.push(event);
    Ok(())
}
