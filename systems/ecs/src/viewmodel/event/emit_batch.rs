//! Emit multiple events in batch

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::Event;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct EmitBatchArgs {
    events: Vec<Event>,
}

/// Emit multiple events in batch
pub fn emit_batch(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: EmitBatchArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Add all events to queue
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.extend(args.events);
        }

        Ok(Vec::new())
    })
}
