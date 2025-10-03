//! Query fragment implementation

use async_trait::async_trait;
use crate::{
    EcsResult, EcsError,
    model::{World, Entity, Component, Query, QueryId, QueryFilter},
    view::query::QueryView,
};

/// Query operations fragment
pub struct QueryFragment;

#[async_trait]
impl QueryView for QueryFragment {
    async fn create_query(&self, _world: &World, _filter: QueryFilter) -> EcsResult<Query> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn execute_query(&self, _world: &World, _query: &Query) -> EcsResult<Vec<Entity>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn execute_query_batch(&self, _world: &World, _query: &Query, _batch_size: usize) -> EcsResult<Vec<Vec<Entity>>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn query_count(&self, _world: &World, _query: &Query) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn delete_query(&self, _world: &World, _query: &Query) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn update_query(&self, _world: &World, _query: &Query, _filter: QueryFilter) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_query(&self, _world: &World, _query_id: QueryId) -> EcsResult<Query> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_all_queries(&self, _world: &World) -> EcsResult<Vec<Query>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn query_has_results(&self, _world: &World, _query: &Query) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn query_first(&self, _world: &World, _query: &Query) -> EcsResult<Entity> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn execute_query_with_components(&self, _world: &World, _query: &Query) -> EcsResult<Vec<(Entity, Vec<Component>)>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn query_entities(&self, _world: &World, _filter: QueryFilter) -> EcsResult<Vec<Entity>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn query_exists(&self, _world: &World, _query_id: QueryId) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clone_query(&self, _world: &World, _query: &Query) -> EcsResult<Query> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }
}