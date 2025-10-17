//! Update a query's filter

use playground_core_ecs::{World, Query, QueryFilter, EcsResult};

/// Update a query's filter
pub async fn update_query(world: &World, query: &Query, filter: QueryFilter) -> EcsResult<()> {
    // Update query in World
    let mut queries = world.queries.write().await;
    queries.insert(query.id, filter);
    Ok(())
}
