//! Get the event queue size

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

/// Get the event queue size
pub fn get_event_queue_size(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get queue size
        let size = {
            let event_queue = world.event_queue.read().await;
            event_queue.len()
        };

        // Serialize result
        let result = bincode::serialize(&size)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
