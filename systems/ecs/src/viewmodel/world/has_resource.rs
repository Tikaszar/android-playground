//! Check if a resource exists

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::World;
use std::pin::Pin;
use std::future::Future;

/// Arguments for has_resource
#[derive(serde::Deserialize)]
struct HasResourceArgs {
    world: World,
    type_name: String,
}

/// Check if a resource exists
pub fn has_resource(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize arguments
        let _args: HasResourceArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let _world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Check if resource exists
        // For now, always return false
        let exists = false;

        // Serialize and return
        let result = bincode::serialize(&exists)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}