//! Component view stub implementation

use async_trait::async_trait;
use playground_modules_types::{ViewFragmentTrait, ViewId, FragmentId};
use crate::{
    EcsResult, EcsError,
    model::{World, Entity, Component, ComponentId},
    view::component::ComponentView,
};

pub const COMPONENT_FRAGMENT_ID: FragmentId = 0x0002;

/// EcsView implementation for ComponentView fragment
pub struct EcsView;

#[async_trait]
impl ViewFragmentTrait for EcsView {
    fn view_id(&self) -> ViewId {
        crate::ECS_VIEW_ID
    }

    fn fragment_id(&self) -> FragmentId {
        COMPONENT_FRAGMENT_ID
    }
}

#[async_trait]
impl ComponentView for EcsView {
    async fn add_component(&self, _world: &World, _entity: Entity, _component: Component) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn add_components(&self, _world: &World, _entity: Entity, _components: Vec<Component>) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn remove_component(&self, _world: &World, _entity: Entity, _component_id: ComponentId) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn remove_components(&self, _world: &World, _entity: Entity, _component_ids: Vec<ComponentId>) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_component(&self, _world: &World, _entity: Entity, _component_id: ComponentId) -> EcsResult<Component> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_components(&self, _world: &World, _entity: Entity, _component_ids: Vec<ComponentId>) -> EcsResult<Vec<Component>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_all_components(&self, _world: &World, _entity: Entity) -> EcsResult<Vec<Component>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn has_component(&self, _world: &World, _entity: Entity, _component_id: ComponentId) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn has_components(&self, _world: &World, _entity: Entity, _component_ids: Vec<ComponentId>) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn replace_component(&self, _world: &World, _entity: Entity, _component: Component) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clear_components(&self, _world: &World, _entity: Entity) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_entities_with_component(&self, _world: &World, _component_id: ComponentId) -> EcsResult<Vec<Entity>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn count_components(&self, _world: &World, _entity: Entity) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_entities_with_components(&self, _world: &World, _component_ids: Vec<ComponentId>) -> EcsResult<Vec<Entity>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }
}