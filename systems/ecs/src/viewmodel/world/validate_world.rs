//! Validate world integrity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::World;
use std::pin::Pin;
use std::future::Future;

/// Validate world integrity
pub fn validate_world(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize world
        let _world_arg: World = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        let mut validation_errors: Vec<String> = Vec::new();

        // Check for invalid entity IDs
        let entities = world.entities.read().await;
        for (entity_id, _generation) in entities.iter() {
            if entity_id.0 == 0 {
                validation_errors.push(format!("Invalid entity ID 0"));
            }
        }

        // Check for invalid query IDs
        let queries = world.queries.read().await;
        for (query_id, _filter) in queries.iter() {
            if query_id.0 == 0 {
                validation_errors.push(format!("Invalid query ID 0"));
            }
        }

        // Additional validation could check:
        // - Dangling entity references in components
        // - Orphaned components without entities
        // - Invalid system dependencies
        // - Circular query dependencies

        // Serialize and return
        let result = bincode::serialize(&validation_errors)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}