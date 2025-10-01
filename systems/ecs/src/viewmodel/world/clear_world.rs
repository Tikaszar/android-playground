//! Clear all entities and components

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;
use std::sync::atomic::Ordering;

/// Clear all entities and components
pub fn clear_world(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get the World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Clear all entities and components
        {
            let mut entities = world.entities.write().await;
            entities.clear();
        }
        {
            let mut components = world.components.write().await;
            components.clear();
        }

        // Reset entity ID counter (starts at 1 again)
        world.next_entity_id.store(1, Ordering::SeqCst);

        // Clear event queue but keep handlers
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.clear();
        }

        Ok(Vec::new())
    })
}