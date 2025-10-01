//! Clear the event queue without processing

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

/// Clear the event queue without processing
pub fn clear_event_queue(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Clear event queue
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.clear();
        }

        Ok(Vec::new())
    })
}
