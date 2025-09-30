//! Entity management API functions

use crate::{
    EcsResult,
    model::{World, Entity, Component},
};

/// Spawn a new entity with components
pub async fn spawn_entity(_world: &World, _components: Vec<Component>) -> EcsResult<Entity> {
    todo!("Implemented by systems/ecs")
}

/// Despawn an entity
pub async fn despawn_entity(_world: &World, _entity: Entity) -> EcsResult<()> {
    todo!("Implemented by systems/ecs")
}

/// Check if an entity exists
pub async fn exists(_world: &World, _entity: Entity) -> EcsResult<bool> {
    todo!("Implemented by systems/ecs")
}

/// Check if an entity is alive (valid generation)
pub async fn is_alive(_world: &World, _entity: Entity) -> EcsResult<bool> {
    todo!("Implemented by systems/ecs")
}

/// Clone an entity with all its components
pub async fn clone_entity(_world: &World, _entity: Entity) -> EcsResult<Entity> {
    todo!("Implemented by systems/ecs")
}