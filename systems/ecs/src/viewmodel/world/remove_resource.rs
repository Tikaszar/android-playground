//! Remove a resource from the world

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::World;
use std::pin::Pin;
use std::future::Future;

/// Arguments for remove_resource
#[derive(serde::Deserialize)]
struct RemoveResourceArgs {
    world: World,
    type_name: String,
}

/// Remove a resource from the world
pub fn remove_resource(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize arguments
        let _args: RemoveResourceArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let _world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Resources would be removed from global singleton components
        // For now, this is a placeholder implementation

        Ok(Vec::new())
    })
}