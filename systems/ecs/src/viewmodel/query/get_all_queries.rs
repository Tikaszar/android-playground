//! Get all queries

use playground_core_ecs::{World, Query, EcsResult};

/// Get all queries
pub async fn get_all_queries(world: &World) -> EcsResult<Vec<Query>> {
    // Get all queries from World
    let queries = world.queries.read().await;
    let mut result = Vec::new();
    for (query_id, filter) in queries.iter() {
        result.push(Query::new(*query_id, filter.clone(), world.clone()));
    }

    Ok(result)
}
