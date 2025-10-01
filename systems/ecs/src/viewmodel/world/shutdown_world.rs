//! Shutdown the World instance

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

/// Shutdown the world
pub fn shutdown_world(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get the World to verify it exists
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Clear all data
        {
            let mut entities = world.entities.write().await;
            entities.clear();
        }
        {
            let mut components = world.components.write().await;
            components.clear();
        }
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.clear();
        }
        {
            let mut pre_handlers = world.pre_handlers.write().await;
            pre_handlers.clear();
        }
        {
            let mut post_handlers = world.post_handlers.write().await;
            post_handlers.clear();
        }

        // Note: We can't actually remove the OnceCell, but data is cleared
        Ok(Vec::new())
    })
}