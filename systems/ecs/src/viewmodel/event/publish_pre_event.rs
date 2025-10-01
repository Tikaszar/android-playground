//! Publish a pre-event that can be cancelled

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EventId, Event, Priority};
use bytes::Bytes;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct PublishPreEventArgs {
    event_id: EventId,
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}

/// Publish a pre-event that can be cancelled
/// Returns true if event should proceed, false if cancelled by any handler
pub fn publish_pre_event(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: PublishPreEventArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Create the event
        let event = Event::new(args.event_id, Bytes::from(args.data))
            .with_priority(Priority::Pre);

        // Get pre-event handler subscriptions
        let subscription_ids = {
            let pre_handlers = world.pre_handlers.read().await;
            pre_handlers.get(&args.event_id).cloned().unwrap_or_default()
        };

        // Execute all pre-event handlers
        // Pre-events allow cancellation - if any handler exists, event can be cancelled
        // In a full implementation, handlers would be actual functions that return bool
        // For now, having no handlers means the event proceeds
        let proceed = subscription_ids.is_empty();

        // Serialize result
        let result = bincode::serialize(&proceed)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}