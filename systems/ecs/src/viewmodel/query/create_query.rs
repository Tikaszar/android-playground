//! Create a new query with a filter

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{QueryId, QueryFilter};
use std::pin::Pin;
use std::future::Future;

/// Create a new query with a filter
pub fn create_query(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize filter
        let filter: QueryFilter = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Generate new query ID
        let query_id = QueryId(world.next_query_id.fetch_add(1));

        // Store query in World
        {
            let mut queries = world.queries.write().await;
            queries.insert(query_id, filter.clone());
        }

        // Create result with Query structure
        #[derive(serde::Serialize)]
        struct CreateQueryResult {
            id: QueryId,
            filter: QueryFilter,
        }

        let result_data = CreateQueryResult {
            id: query_id,
            filter,
        };

        // Serialize and return
        let result = bincode::serialize(&result_data)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
