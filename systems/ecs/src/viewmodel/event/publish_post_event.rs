//! Publish a post-event (for post-processing hooks)

use playground_core_ecs::{World, Event, Priority, EcsResult};

/// Publish a post-event (for post-processing hooks)
pub async fn publish_post_event(world: &World, event: Event) -> EcsResult<()> {
    // Create event with Normal priority (post-events are notifications)
    let event = event.with_priority(Priority::Normal);

    // Add to event queue for deferred processing
    let mut event_queue = world.event_queue.write().await;
    event_queue.push(event);

    Ok(())
}
