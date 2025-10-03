//! Component view trait - API contract only

use async_trait::async_trait;
use crate::{
    EcsResult,
    model::{World, Entity, Component, ComponentId},
};

/// Component management API contract
#[async_trait]
pub trait ComponentView: Send + Sync {
    /// Add a component to an entity
    async fn add_component(&self, world: &World, entity: Entity, component: Component) -> EcsResult<()>;

    /// Add multiple components to an entity in batch
    async fn add_components(&self, world: &World, entity: Entity, components: Vec<Component>) -> EcsResult<()>;

    /// Remove a component from an entity
    async fn remove_component(&self, world: &World, entity: Entity, component_id: ComponentId) -> EcsResult<()>;

    /// Remove multiple components from an entity in batch
    async fn remove_components(&self, world: &World, entity: Entity, component_ids: Vec<ComponentId>) -> EcsResult<()>;

    /// Get a component from an entity
    async fn get_component(&self, world: &World, entity: Entity, component_id: ComponentId) -> EcsResult<Component>;

    /// Get multiple specific components from an entity
    async fn get_components(&self, world: &World, entity: Entity, component_ids: Vec<ComponentId>) -> EcsResult<Vec<Component>>;

    /// Get all components for an entity
    async fn get_all_components(&self, world: &World, entity: Entity) -> EcsResult<Vec<Component>>;

    /// Check if an entity has a component
    async fn has_component(&self, world: &World, entity: Entity, component_id: ComponentId) -> EcsResult<bool>;

    /// Check if an entity has all specified components
    async fn has_components(&self, world: &World, entity: Entity, component_ids: Vec<ComponentId>) -> EcsResult<bool>;

    /// Replace a component on an entity (add or update)
    async fn replace_component(&self, world: &World, entity: Entity, component: Component) -> EcsResult<()>;

    /// Clear all components from an entity
    async fn clear_components(&self, world: &World, entity: Entity) -> EcsResult<()>;

    /// Get all entities that have a specific component
    async fn get_entities_with_component(&self, world: &World, component_id: ComponentId) -> EcsResult<Vec<Entity>>;

    /// Count components on an entity
    async fn count_components(&self, world: &World, entity: Entity) -> EcsResult<usize>;

    /// Get all entities that have all specified components
    async fn get_entities_with_components(&self, world: &World, component_ids: Vec<ComponentId>) -> EcsResult<Vec<Entity>>;
}