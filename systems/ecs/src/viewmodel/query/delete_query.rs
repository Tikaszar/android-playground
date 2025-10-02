//! Delete a query

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::Query;
use std::pin::Pin;
use std::future::Future;

/// Delete a query
pub fn delete_query(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize query
        let query: Query = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Remove query from World
        {
            let mut queries = world.queries.write().await;
            queries.remove(&query.id);
        }

        // Return empty success
        Ok(vec![])
    })
}