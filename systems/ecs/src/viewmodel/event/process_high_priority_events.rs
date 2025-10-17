//! Process only high priority events

use playground_core_ecs::{World, Priority, EcsResult};

/// Process only high priority events
/// Returns the number of events processed
pub async fn process_high_priority_events(world: &World) -> EcsResult<usize> {
    // Extract high priority events for processing
    let mut event_queue = world.event_queue.write().await;

    // Separate high priority events from others
    let (high_priority, other_priority): (Vec<_>, Vec<_>) = event_queue
        .drain(..)
        .partition(|event| matches!(event.priority, Priority::High | Priority::Critical));

    // Put back non-high priority events
    *event_queue = other_priority;
    drop(event_queue);

    let mut processed_count = 0;

    // Process each high priority event
    for event in high_priority {
        // Get handlers for this event
        let pre_handlers = {
            let handlers = world.pre_handlers.read().await;
            handlers.get(&event.id).cloned().unwrap_or_default()
        };

        let post_handlers = {
            let handlers = world.post_handlers.read().await;
            handlers.get(&event.id).cloned().unwrap_or_default()
        };

        // Count handlers that would be notified
        let handler_count = pre_handlers.len() + post_handlers.len();

        if handler_count > 0 {
            processed_count += 1;
        }
    }

    Ok(processed_count)
}
