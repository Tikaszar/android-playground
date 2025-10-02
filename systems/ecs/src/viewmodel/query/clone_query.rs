//! Clone a query with a new ID

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{Query, QueryId};
use std::pin::Pin;
use std::future::Future;

/// Clone a query with a new ID
pub fn clone_query(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize query
        let query: Query = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get filter from original query
        let filter = {
            let queries = world.queries.read().await;
            queries.get(&query.id)
                .ok_or_else(|| ModuleError::Generic(format!("Query {:?} not found", query.id)))?
                .clone()
        };

        // Generate new query ID
        let new_query_id = QueryId(world.next_query_id.fetch_add(1));

        // Store cloned query
        {
            let mut queries = world.queries.write().await;
            queries.insert(new_query_id, filter.clone());
        }

        // Create new Query object
        let cloned_query = Query::new(new_query_id, filter, world.clone());

        // Serialize and return
        let result = bincode::serialize(&cloned_query)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}