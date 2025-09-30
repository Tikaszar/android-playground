//! Component management API functions

use crate::{
    EcsResult,
    model::{World, Entity, Component, ComponentId},
};

/// Add a component to an entity
pub async fn add_component(
    _world: &World,
    _entity: Entity,
    _component: Component,
) -> EcsResult<()> {
    todo!("Implemented by systems/ecs")
}

/// Remove a component from an entity
pub async fn remove_component(
    _world: &World,
    _entity: Entity,
    _component_id: ComponentId,
) -> EcsResult<()> {
    todo!("Implemented by systems/ecs")
}

/// Get a component from an entity
pub async fn get_component(
    _world: &World,
    _entity: Entity,
    _component_id: ComponentId,
) -> EcsResult<Component> {
    todo!("Implemented by systems/ecs")
}

/// Check if an entity has a component
pub async fn has_component(
    _world: &World,
    _entity: Entity,
    _component_id: ComponentId,
) -> EcsResult<bool> {
    todo!("Implemented by systems/ecs")
}

/// Get all components for an entity
pub async fn get_all_components(
    _world: &World,
    _entity: Entity,
) -> EcsResult<Vec<Component>> {
    todo!("Implemented by systems/ecs")
}