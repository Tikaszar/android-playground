//! Publish a post-event (notification after state change)

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EventId, Event, Priority};
use bytes::Bytes;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct PublishPostEventArgs {
    event_id: EventId,
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}

/// Publish a post-event (notification after state change)
/// Post-events are added to the queue for asynchronous processing
pub fn publish_post_event(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: PublishPostEventArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Create event with Normal priority (post-events are notifications)
        let event = Event::new(args.event_id, Bytes::from(args.data))
            .with_priority(Priority::Normal);

        // Add to event queue for deferred processing
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.push(event);
        }

        // Get post-event handler subscriptions for metrics
        let subscription_count = {
            let post_handlers = world.post_handlers.read().await;
            post_handlers.get(&args.event_id).map_or(0, |v| v.len())
        };

        // Serialize subscription count as confirmation
        let result = bincode::serialize(&subscription_count)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}