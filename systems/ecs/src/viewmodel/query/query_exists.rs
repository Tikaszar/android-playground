//! Check if a query exists

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::QueryId;
use std::pin::Pin;
use std::future::Future;

/// Check if a query exists
pub fn query_exists(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize query ID
        let query_id: QueryId = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Check if query exists
        let exists = {
            let queries = world.queries.read().await;
            queries.contains_key(&query_id)
        };

        // Serialize and return
        let result = bincode::serialize(&exists)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}