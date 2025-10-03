//! Entity fragment implementation

use async_trait::async_trait;
use crate::{
    EcsResult, EcsError,
    model::{World, Entity, EntityId, Generation, Component},
    view::entity::EntityView,
};

/// Entity operations fragment
pub struct EntityFragment;

#[async_trait]
impl EntityView for EntityFragment {
    async fn spawn_entity(&self, _world: &World, _components: Vec<Component>) -> EcsResult<Entity> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn spawn_batch(&self, _world: &World, _batches: Vec<Vec<Component>>) -> EcsResult<Vec<Entity>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn despawn_entity(&self, _world: &World, _entity: Entity) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn despawn_batch(&self, _world: &World, _entities: Vec<Entity>) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clone_entity(&self, _world: &World, _entity: Entity) -> EcsResult<Entity> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn exists(&self, _world: &World, _entity: Entity) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn is_alive(&self, _world: &World, _entity: Entity) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_all_entities(&self, _world: &World) -> EcsResult<Vec<Entity>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_entity_count(&self, _world: &World) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_entity(&self, _world: &World, _entity_id: EntityId) -> EcsResult<Entity> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_generation(&self, _world: &World, _entity_id: EntityId) -> EcsResult<Generation> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn spawn_entity_with_id(&self, _world: &World, _entity_id: EntityId, _components: Vec<Component>) -> EcsResult<Entity> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }
}