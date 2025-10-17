//! Create a new query with a filter

use playground_core_ecs::{World, Query, QueryFilter, QueryId, EcsResult};

/// Create a new query with a filter
pub async fn create_query(world: &World, filter: QueryFilter) -> EcsResult<Query> {
    // Generate new query ID
    let query_id = QueryId(world.next_query_id.fetch_add(1));

    // Store query in World
    let mut queries = world.queries.write().await;
    queries.insert(query_id, filter.clone());

    // Create and return Query
    let query = Query {
        id: query_id,
        filter,
    };

    Ok(query)
}
