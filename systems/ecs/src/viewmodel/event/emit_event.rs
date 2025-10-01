//! Emit an event to be processed

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{Event, EventId, EntityRef, EntityId, Generation, Priority};
use bytes::Bytes;
use std::pin::Pin;
use std::future::Future;
use std::sync::Weak;

#[derive(serde::Deserialize)]
struct EmitEventArgs {
    event_id: EventId,
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
    source_id: Option<EntityId>,
    source_generation: Option<Generation>,
    priority: Option<Priority>,
}

/// Emit an event to be processed
pub fn emit_event(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: EmitEventArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Create event
        let mut event = Event::new(args.event_id, Bytes::from(args.data));

        // Set source if provided
        if let (Some(id), Some(generation)) = (args.source_id, args.source_generation) {
            event.source = EntityRef {
                id,
                generation,
                world: Weak::new(),
            };
        }

        // Set priority if provided
        if let Some(priority) = args.priority {
            event.priority = priority;
        }

        // Add to event queue
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.push(event);
        }

        Ok(Vec::new())
    })
}
