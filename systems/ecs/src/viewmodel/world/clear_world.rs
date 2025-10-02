//! Clear all entities and components

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

/// Clear all entities and components
pub fn clear_world(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get the World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Clear all entities
        {
            let mut entities = world.entities.write().await;
            entities.clear();
        }

        // Clear component registry
        {
            let mut component_registry = world.component_registry.write().await;
            component_registry.clear();
        }

        // Reset entity ID counter (starts at 1 again)
        world.next_entity_id.store(1);

        // Clear event queue but keep handlers
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.clear();
        }

        // Clear queries
        {
            let mut queries = world.queries.write().await;
            queries.clear();
        }

        // Reset query ID counter
        world.next_query_id.store(1);

        // Clear storages
        {
            let mut storages = world.storages.write().await;
            storages.clear();
        }

        // Reset storage ID counter
        world.next_storage_id.store(1);

        // Clear systems
        {
            let mut systems = world.systems.write().await;
            systems.clear();
        }

        // Reset system ID counter
        world.next_system_id.store(1);

        // Reset subscription ID counter
        world.next_subscription_id.store(1);

        Ok(Vec::new())
    })
}