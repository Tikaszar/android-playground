//! Emit multiple events in batch

use playground_core_ecs::{World, Event, EcsResult};

/// Emit multiple events in batch
pub async fn emit_batch(world: &World, events: Vec<Event>) -> EcsResult<()> {
    // Add all events to queue
    let mut event_queue = world.event_queue.write().await;
    event_queue.extend(events);
    Ok(())
}
