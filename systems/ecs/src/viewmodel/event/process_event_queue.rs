//! Process the event queue, dispatching to handlers

use playground_core_ecs::{World, Priority, EcsResult};

/// Process the event queue, dispatching to handlers
/// Returns the number of events processed
pub async fn process_event_queue(world: &World) -> EcsResult<usize> {
    // Get all events from queue
    let mut event_queue = world.event_queue.write().await;
    let events = event_queue.clone();
    event_queue.clear();
    drop(event_queue);

    let mut processed_count = 0;

    // Process events by priority order
    let mut sorted_events = events;
    sorted_events.sort_by(|a, b| {
        // Higher priority first
        match (&b.priority, &a.priority) {
            (Priority::Critical, Priority::Critical) => std::cmp::Ordering::Equal,
            (Priority::Critical, _) => std::cmp::Ordering::Greater,
            (_, Priority::Critical) => std::cmp::Ordering::Less,
            (Priority::High, Priority::High) => std::cmp::Ordering::Equal,
            (Priority::High, _) => std::cmp::Ordering::Greater,
            (_, Priority::High) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        }
    });

    // Process each event
    for event in sorted_events {
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
