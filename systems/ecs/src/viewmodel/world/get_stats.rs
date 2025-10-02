//! Get world statistics

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::WorldStats;
use std::pin::Pin;
use std::future::Future;

/// Get world statistics
pub fn get_stats(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get World from global state
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Gather statistics
        let entity_count = {
            let entities = world.entities.read().await;
            entities.len()
        };

        let query_count = {
            let queries = world.queries.read().await;
            queries.len()
        };

        let event_queue_size = {
            let event_queue = world.event_queue.read().await;
            event_queue.len()
        };

        let subscription_count = {
            let subscriptions = world.subscriptions.read().await;
            subscriptions.len()
        };

        let storage_count = {
            let storages = world.storages.read().await;
            storages.len()
        };

        let system_count = {
            let systems = world.systems.read().await;
            systems.len()
        };

        // Create stats structure
        let stats = WorldStats {
            entity_count,
            component_count: 0,  // Components are in System.component_pools now
            system_count,
            query_count,
            event_queue_size,
            subscription_count,
            storage_count,
        };

        // Serialize and return
        let result = bincode::serialize(&stats)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}