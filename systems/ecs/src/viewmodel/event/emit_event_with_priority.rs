//! Emit an event with specific priority

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{Event, EventId, Priority};
use bytes::Bytes;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct EmitEventWithPriorityArgs {
    event_id: EventId,
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
    priority: Priority,
}

/// Emit an event with specific priority
pub fn emit_event_with_priority(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: EmitEventWithPriorityArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Create event with priority
        let event = Event::new(args.event_id, Bytes::from(args.data))
            .with_priority(args.priority);

        // Add to event queue
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.push(event);
        }

        Ok(Vec::new())
    })
}
