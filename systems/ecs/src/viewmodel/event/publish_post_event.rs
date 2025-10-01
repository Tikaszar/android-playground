//! Publish a post-event (notification after state change)

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EventId, Event};
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
pub fn publish_post_event(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize args
        let args: PublishPostEventArgs = bincode::deserialize(args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Create event with Bytes
        let event = Event::new(args.event_id, Bytes::from(args.data));

        // Add to event queue for processing
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.push(event);
        }

        // Get post-event handlers for this event
        let _handler_ids = {
            let post_handlers = world.post_handlers.read().await;
            post_handlers.get(&args.event_id).cloned().unwrap_or_default()
        };

        // For now, we don't actually execute handlers (would need handler registry)

        Ok(Vec::new())
    })
}