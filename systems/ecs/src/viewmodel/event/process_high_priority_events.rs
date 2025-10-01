//! Process only high priority events

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::Priority;
use std::pin::Pin;
use std::future::Future;

/// Process only high priority events
/// Returns the number of events processed
pub fn process_high_priority_events(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Extract high priority events for processing
        let high_priority_events = {
            let mut event_queue = world.event_queue.write().await;

            // Separate high priority events from others
            let (high_priority, other_priority): (Vec<_>, Vec<_>) = event_queue
                .drain(..)
                .partition(|event| matches!(event.priority, Priority::High | Priority::Critical));

            // Put back non-high priority events
            *event_queue = other_priority;

            high_priority
        };

        let mut processed_count = 0;

        // Process each high priority event
        for event in high_priority_events {
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

        // Serialize result
        let result = bincode::serialize(&processed_count)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
