//! Get the World instance

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

/// Get the world instance
pub fn get_world(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Get the World handle
        let _world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Return empty success - world accessible via API
        // (We don't serialize the whole world, just confirm it exists)
        Ok(Vec::new())
    })
}