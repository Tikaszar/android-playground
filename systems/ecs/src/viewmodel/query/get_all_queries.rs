//! Get all queries

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::Query;
use std::pin::Pin;
use std::future::Future;

/// Get all queries
pub fn get_all_queries(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let _args = args.to_vec();
    Box::pin(async move {
        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get all queries from World
        let all_queries = {
            let queries = world.queries.read().await;
            let mut result = Vec::new();
            for (query_id, filter) in queries.iter() {
                result.push(Query::new(*query_id, filter.clone(), world.clone()));
            }
            result
        };

        // Serialize and return
        let result = bincode::serialize(&all_queries)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}