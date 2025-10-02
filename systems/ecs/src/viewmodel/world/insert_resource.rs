//! Insert a resource into the world

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::World;
use bytes::Bytes;
use std::pin::Pin;
use std::future::Future;

/// Arguments for insert_resource
#[derive(serde::Deserialize)]
struct InsertResourceArgs {
    world: World,
    type_name: String,
    data: Bytes,
}

/// Insert a resource into the world
pub fn insert_resource(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize arguments
        let _args: InsertResourceArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let _world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Resources would be stored as global singleton components
        // For now, this is a placeholder implementation

        Ok(Vec::new())
    })
}