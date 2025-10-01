//! Get the total entity count

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

/// Get the total entity count
pub fn get_entity_count(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get the World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get entity count
        let count = {
            let entities = world.entities.read().await;
            entities.len()
        };

        // Serialize count
        let result = bincode::serialize(&count)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}