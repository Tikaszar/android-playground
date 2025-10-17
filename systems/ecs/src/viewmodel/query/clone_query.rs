//! Clone a query with a new ID

use playground_core_ecs::{World, Query, QueryId, EcsResult, EcsError};

/// Clone a query with a new ID
pub async fn clone_query(world: &World, query: &Query) -> EcsResult<Query> {
    // Get filter from original query
    let queries = world.queries.read().await;
    let filter = queries
        .get(&query.id)
        .ok_or_else(|| EcsError::QueryNotFound(query.id))?
        .clone();
    drop(queries);

    // Generate new query ID
    let new_query_id = QueryId(world.next_query_id.fetch_add(1));

    // Store cloned query
    let mut queries = world.queries.write().await;
    queries.insert(new_query_id, filter.clone());
    drop(queries);

    // Create new Query object
    let cloned_query = Query::new(new_query_id, filter, world.clone());

    Ok(cloned_query)
}
