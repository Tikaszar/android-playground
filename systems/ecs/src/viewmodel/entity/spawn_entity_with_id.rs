//! Spawn entity with specific ID (useful for deserialization)

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Generation};
use std::pin::Pin;
use std::future::Future;

/// Spawn entity with specific ID (useful for deserialization)
pub fn spawn_entity_with_id(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize arguments
        #[derive(serde::Deserialize)]
        struct SpawnWithIdArgs {
            entity_id: EntityId,
        }

        let spawn_args: SpawnWithIdArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        let entity_id = spawn_args.entity_id;
        let generation = Generation(1);

        // Check if entity already exists
        {
            let entities = world.entities.read().await;
            if entities.contains_key(&entity_id) {
                return Err(ModuleError::Generic(format!("Entity {:?} already exists", entity_id)));
            }
        }

        // Store entity in World
        {
            let mut entities = world.entities.write().await;
            entities.insert(entity_id, generation);
        }

        // Update next_entity_id if needed
        {
            use std::sync::atomic::Ordering;
            let current_next = world.next_entity_id.load(Ordering::SeqCst);
            if entity_id.0 >= current_next {
                world.next_entity_id.store(entity_id.0 + 1, Ordering::SeqCst);
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
