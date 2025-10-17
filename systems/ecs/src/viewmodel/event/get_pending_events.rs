//! Get pending events without processing them

use playground_core_ecs::{World, Event, EcsResult};

/// Get pending events without processing them
pub async fn get_pending_events(world: &World) -> EcsResult<Vec<Event>> {
    // Get events (clone them, don't remove)
    let event_queue = world.event_queue.read().await;
    let events = event_queue.clone();
    Ok(events)
}
