//! Get a query by ID

use playground_core_ecs::{World, QueryId, Query, EcsResult, EcsError};

/// Get a query by ID
pub async fn get_query(world: &World, query_id: QueryId) -> EcsResult<Query> {
    // Get query from World
    let queries = world.queries.read().await;
    let filter = queries
        .get(&query_id)
        .ok_or_else(|| EcsError::QueryNotFound(query_id))?
        .clone();

    // Create Query object
    let query = Query::new(query_id, filter, world.clone());

    Ok(query)
}
