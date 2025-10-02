//! Get a resource from the world

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::World;
use bytes::Bytes;
use std::pin::Pin;
use std::future::Future;

/// Arguments for get_resource
#[derive(serde::Deserialize)]
struct GetResourceArgs {
    world: World,
    type_name: String,
}

/// Get a resource from the world
pub fn get_resource(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize arguments
        let args: GetResourceArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let _world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Resources would be retrieved from global singleton components
        // For now, return empty bytes as placeholder
        let resource_data = Bytes::new();

        // Serialize and return
        let result = bincode::serialize(&resource_data)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}