//! Update a query's filter

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{Query, QueryFilter};
use std::pin::Pin;
use std::future::Future;

/// Arguments for update_query
#[derive(serde::Deserialize)]
struct UpdateQueryArgs {
    query: Query,
    filter: QueryFilter,
}

/// Update a query's filter
pub fn update_query(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize arguments
        let args: UpdateQueryArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Update query in World
        {
            let mut queries = world.queries.write().await;
            queries.insert(args.query.id, args.filter);
        }

        // Return empty success
        Ok(vec![])
    })
}