//! Spawn a new entity with components

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Generation, Component};
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;

/// Spawn a new entity with components
pub fn spawn_entity(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize components
        let components: Vec<Component> = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Generate new entity ID
        let entity_id = EntityId(world.next_entity_id.fetch_add(1));
        let generation = Generation(1);

        // Store entity in World
        {
            let mut entities = world.entities.write().await;
            entities.insert(entity_id, generation);
        }

        // Store components
        // Components are now stored in System.component_pools after Session 77
        // This would need to:
        // 1. Look up which system owns each component type via world.component_registry
        // 2. Access that system's component_pools
        // 3. Store the component in the appropriate pool
        // For now, we skip component storage as the architecture is being updated
        if !components.is_empty() {
            // Real implementation would store in System.component_pools
        }

        // Create result with IDs only
        #[derive(serde::Serialize)]
        struct SpawnEntityResult {
            id: EntityId,
            generation: Generation,
        }

        let result_data = SpawnEntityResult {
            id: entity_id,
            generation,
        };

        // Serialize and return
        let result = bincode::serialize(&result_data)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}