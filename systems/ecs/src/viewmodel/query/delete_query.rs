//! Delete a query

use playground_core_ecs::{World, Query, EcsResult};

/// Delete a query
pub async fn delete_query(world: &World, query: &Query) -> EcsResult<()> {
    // Remove query from World
    let mut queries = world.queries.write().await;
    queries.remove(&query.id);
    Ok(())
}
