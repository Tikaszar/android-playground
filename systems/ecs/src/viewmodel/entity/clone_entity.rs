//! Clone an entity with all its components

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Generation};
use std::pin::Pin;
use std::future::Future;
use std::sync::atomic::Ordering;

#[derive(serde::Serialize)]
struct CloneEntityResult {
    id: EntityId,
    generation: Generation,
}

/// Clone an entity with all its components
pub fn clone_entity(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize source entity ID from args
        let source_entity_id: EntityId = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Verify source entity exists
        let exists = {
            let entities = world.entities.read().await;
            entities.contains_key(&source_entity_id)
        };

        if !exists {
            return Err(ModuleError::Generic(format!("Entity {:?} not found", source_entity_id)));
        }

        // Generate new entity ID
        let new_entity_id = EntityId(world.next_entity_id.fetch_add(1, Ordering::SeqCst));
        let generation = Generation(1);

        // Store new entity
        {
            let mut entities = world.entities.write().await;
            entities.insert(new_entity_id, generation);
        }

        // Clone components
        {
            let components = world.components.read().await;
            if let Some(source_components) = components.get(&source_entity_id) {
                let cloned_components = source_components.clone();
                drop(components); // Release read lock

                let mut components = world.components.write().await;
                components.insert(new_entity_id, cloned_components);
            }
        }

        // Create result with IDs only
        let result_data = CloneEntityResult {
            id: new_entity_id,
            generation,
        };

        // Serialize and return
        let result = bincode::serialize(&result_data)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}