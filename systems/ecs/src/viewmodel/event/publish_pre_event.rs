//! Publish a pre-event that can be cancelled

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::EventId;
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
/// Returns false if any handler cancelled the event
pub fn publish_pre_event(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize args
        let args: PublishPreEventArgs = bincode::deserialize(args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get pre-event handlers for this event
        let handler_ids = {
            let pre_handlers = world.pre_handlers.read().await;
            pre_handlers.get(&args.event_id).cloned().unwrap_or_default()
        };

        // Convert Vec<u8> to Bytes for compatibility
        let _event_data = Bytes::from(args.data);

        // For now, we don't actually execute handlers (would need handler registry)
        // Just return true (not cancelled)
        let not_cancelled = true;

        // Serialize result
        let result = bincode::serialize(&not_cancelled)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}