//! Check if a query exists

use playground_core_ecs::{World, QueryId, EcsResult};

/// Check if a query exists
pub async fn query_exists(world: &World, query_id: QueryId) -> EcsResult<bool> {
    // Check if query exists
    let queries = world.queries.read().await;
    let exists = queries.contains_key(&query_id);
    Ok(exists)
}
