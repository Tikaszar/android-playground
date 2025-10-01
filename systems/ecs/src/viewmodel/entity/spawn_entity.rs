//! Spawn a new entity with components

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Generation, Component};
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use std::sync::atomic::Ordering;

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
        let entity_id = EntityId(world.next_entity_id.fetch_add(1, Ordering::SeqCst));
        let generation = Generation(1);

        // Store entity in World
        {
            let mut entities = world.entities.write().await;
            entities.insert(entity_id, generation);
        }

        // Store components
        if !components.is_empty() {
            let mut components_map = world.components.write().await;
            let entity_components = components_map.entry(entity_id).or_insert_with(HashMap::new);
            for component in components {
                entity_components.insert(component.component_id, component);
            }
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