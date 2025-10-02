//! Get a query by ID

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{QueryId, Query};
use std::pin::Pin;
use std::future::Future;

/// Get a query by ID
pub fn get_query(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize query ID
        let query_id: QueryId = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get query from World
        let filter = {
            let queries = world.queries.read().await;
            queries.get(&query_id)
                .ok_or_else(|| ModuleError::Generic(format!("Query {:?} not found", query_id)))?
                .clone()
        };

        // Create Query object
        let query = Query::new(query_id, filter, world.clone());

        // Serialize and return
        let result = bincode::serialize(&query)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}