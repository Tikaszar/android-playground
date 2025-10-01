//! Get pending events without processing them

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::Event;
use std::pin::Pin;
use std::future::Future;

/// Get pending events without processing them
pub fn get_pending_events(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get events (clone them, don't remove)
        let events: Vec<Event> = {
            let event_queue = world.event_queue.read().await;
            event_queue.clone()
        };

        // Serialize result
        let result = bincode::serialize(&events)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
