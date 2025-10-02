//! Reset the world to initial state

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::World;
use std::pin::Pin;
use std::future::Future;

/// Reset the world to initial state
pub fn reset_world(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize world
        let _world_arg: World = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Clear entities
        {
            let mut entities = world.entities.write().await;
            entities.clear();
        }

        // Clear component registry
        {
            let mut component_registry = world.component_registry.write().await;
            component_registry.clear();
        }

        // Reset all counters
        world.next_entity_id.store(1);
        world.next_query_id.store(1);
        world.next_storage_id.store(1);
        world.next_system_id.store(1);
        world.next_subscription_id.store(1);

        // Clear event queue
        {
            let mut event_queue = world.event_queue.write().await;
            event_queue.clear();
        }

        // Keep handlers, queries, systems intact (structure preserved)

        Ok(Vec::new())
    })
}