//! Component management API functions
//!
//! This module provides the View layer API contracts for component operations.
//! These functions are stubs that will be replaced by the actual implementations
//! from systems/ecs at compile time through conditional compilation.

use crate::{
    EcsResult, EcsError,
    model::{World, Entity, Component, ComponentId},
};

/// Add a component to an entity
pub async fn add_component(
    _world: &World,
    _entity: Entity,
    _component: Component,
) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("add_component not implemented - systems/ecs required".to_string()))
}

/// Add multiple components to an entity in batch
pub async fn add_components(
    _world: &World,
    _entity: Entity,
    _components: Vec<Component>,
) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("add_components not implemented - systems/ecs required".to_string()))
}

/// Remove a component from an entity
pub async fn remove_component(
    _world: &World,
    _entity: Entity,
    _component_id: ComponentId,
) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("remove_component not implemented - systems/ecs required".to_string()))
}

/// Remove multiple components from an entity in batch
pub async fn remove_components(
    _world: &World,
    _entity: Entity,
    _component_ids: Vec<ComponentId>,
) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("remove_components not implemented - systems/ecs required".to_string()))
}

/// Get a component from an entity
pub async fn get_component(
    _world: &World,
    _entity: Entity,
    _component_id: ComponentId,
) -> EcsResult<Component> {
    Err(EcsError::ModuleNotFound("get_component not implemented - systems/ecs required".to_string()))
}

/// Get multiple specific components from an entity
pub async fn get_components(
    _world: &World,
    _entity: Entity,
    _component_ids: Vec<ComponentId>,
) -> EcsResult<Vec<Component>> {
    Err(EcsError::ModuleNotFound("get_components not implemented - systems/ecs required".to_string()))
}

/// Get all components for an entity
pub async fn get_all_components(
    _world: &World,
    _entity: Entity,
) -> EcsResult<Vec<Component>> {
    Err(EcsError::ModuleNotFound("get_all_components not implemented - systems/ecs required".to_string()))
}

/// Check if an entity has a component
pub async fn has_component(
    _world: &World,
    _entity: Entity,
    _component_id: ComponentId,
) -> EcsResult<bool> {
    Err(EcsError::ModuleNotFound("has_component not implemented - systems/ecs required".to_string()))
}

/// Check if an entity has all specified components
pub async fn has_components(
    _world: &World,
    _entity: Entity,
    _component_ids: Vec<ComponentId>,
) -> EcsResult<bool> {
    Err(EcsError::ModuleNotFound("has_components not implemented - systems/ecs required".to_string()))
}

/// Replace a component on an entity (add or update)
pub async fn replace_component(
    _world: &World,
    _entity: Entity,
    _component: Component,
) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("replace_component not implemented - systems/ecs required".to_string()))
}

/// Clear all components from an entity
pub async fn clear_components(
    _world: &World,
    _entity: Entity,
) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("clear_components not implemented - systems/ecs required".to_string()))
}

/// Get all entities that have a specific component
pub async fn get_entities_with_component(
    _world: &World,
    _component_id: ComponentId,
) -> EcsResult<Vec<Entity>> {
    Err(EcsError::ModuleNotFound("get_entities_with_component not implemented - systems/ecs required".to_string()))
}

/// Count components on an entity
pub async fn count_components(
    _world: &World,
    _entity: Entity,
) -> EcsResult<usize> {
    Err(EcsError::ModuleNotFound("count_components not implemented - systems/ecs required".to_string()))
}

/// Get all entities that have all specified components
pub async fn get_entities_with_components(
    _world: &World,
    _component_ids: Vec<ComponentId>,
) -> EcsResult<Vec<Entity>> {
    Err(EcsError::ModuleNotFound("get_entities_with_components not implemented - systems/ecs required".to_string()))
}