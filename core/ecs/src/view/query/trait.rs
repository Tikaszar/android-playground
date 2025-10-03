//! Query view trait - API contract only

use async_trait::async_trait;
use crate::{
    EcsResult,
    model::{World, Entity, Component, Query, QueryId, QueryFilter},
};

/// Query system API contract
#[async_trait]
pub trait QueryView: Send + Sync {
    /// Create a new query with a filter
    async fn create_query(&self, world: &World, filter: QueryFilter) -> EcsResult<Query>;

    /// Execute a query and return matching entities
    async fn execute_query(&self, world: &World, query: &Query) -> EcsResult<Vec<Entity>>;

    /// Execute a query and return matching entities in batches
    async fn execute_query_batch(&self, world: &World, query: &Query, batch_size: usize) -> EcsResult<Vec<Vec<Entity>>>;

    /// Get the count of entities matching a query
    async fn query_count(&self, world: &World, query: &Query) -> EcsResult<usize>;

    /// Delete a query
    async fn delete_query(&self, world: &World, query: &Query) -> EcsResult<()>;

    /// Update a query's filter
    async fn update_query(&self, world: &World, query: &Query, filter: QueryFilter) -> EcsResult<()>;

    /// Get a query by ID
    async fn get_query(&self, world: &World, query_id: QueryId) -> EcsResult<Query>;

    /// Get all queries
    async fn get_all_queries(&self, world: &World) -> EcsResult<Vec<Query>>;

    /// Check if any entities match a query
    async fn query_has_results(&self, world: &World, query: &Query) -> EcsResult<bool>;

    /// Get first entity matching a query
    async fn query_first(&self, world: &World, query: &Query) -> EcsResult<Entity>;

    /// Execute query and get entities with their components
    async fn execute_query_with_components(&self, world: &World, query: &Query) -> EcsResult<Vec<(Entity, Vec<Component>)>>;

    /// Create and execute a query in one operation
    async fn query_entities(&self, world: &World, filter: QueryFilter) -> EcsResult<Vec<Entity>>;

    /// Check if a query exists
    async fn query_exists(&self, world: &World, query_id: QueryId) -> EcsResult<bool>;

    /// Clone a query with a new ID
    async fn clone_query(&self, world: &World, query: &Query) -> EcsResult<Query>;
}