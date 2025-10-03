//! Entity view stub implementation

use async_trait::async_trait;
use playground_modules_types::{ViewFragmentTrait, ViewId, FragmentId};
use crate::{
    EcsResult, EcsError,
    model::{World, Entity, EntityId, Generation, Component},
    view::{entity::EntityView, EcsView},
};

pub const ENTITY_FRAGMENT_ID: FragmentId = 0x0001;

#[async_trait]
impl ViewFragmentTrait for EcsView {
    fn view_id(&self) -> ViewId {
        crate::ECS_VIEW_ID
    }

    fn fragment_id(&self) -> FragmentId {
        ENTITY_FRAGMENT_ID
    }
}

#[async_trait]
impl EntityView for EcsView {
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