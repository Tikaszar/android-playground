//! Entity management API functions
//!
//! This module provides the View layer API contracts for entity operations.
//! These functions are stubs that will be replaced by the actual implementations
//! from systems/ecs at compile time through conditional compilation.

use crate::{
    EcsResult, EcsError,
    model::{World, Entity, Component},
};

// These are the API contracts - the actual implementation comes from systems/ecs
// When compiling with the systems/ecs feature, these get replaced by the real implementations

/// Spawn a new entity with components
pub async fn spawn_entity(_world: &World, _components: Vec<Component>) -> EcsResult<Entity> {
    // This stub is replaced by systems/ecs implementation at compile time
    Err(EcsError::ModuleNotFound("spawn_entity not implemented - systems/ecs required".to_string()))
}

/// Spawn multiple entities in batch for efficiency
pub async fn spawn_batch(_world: &World, _batches: Vec<Vec<Component>>) -> EcsResult<Vec<Entity>> {
    Err(EcsError::ModuleNotFound("spawn_batch not implemented - systems/ecs required".to_string()))
}

/// Despawn an entity
pub async fn despawn_entity(_world: &World, _entity: Entity) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("despawn_entity not implemented - systems/ecs required".to_string()))
}

/// Despawn multiple entities in batch
pub async fn despawn_batch(_world: &World, _entities: Vec<Entity>) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("despawn_batch not implemented - systems/ecs required".to_string()))
}

/// Clone an entity with all its components
pub async fn clone_entity(_world: &World, _entity: Entity) -> EcsResult<Entity> {
    Err(EcsError::ModuleNotFound("clone_entity not implemented - systems/ecs required".to_string()))
}

/// Check if an entity exists (regardless of generation)
pub async fn exists(_world: &World, _entity: Entity) -> EcsResult<bool> {
    Err(EcsError::ModuleNotFound("exists not implemented - systems/ecs required".to_string()))
}

/// Check if an entity is alive (valid generation)
pub async fn is_alive(_world: &World, _entity: Entity) -> EcsResult<bool> {
    Err(EcsError::ModuleNotFound("is_alive not implemented - systems/ecs required".to_string()))
}

/// Get all entities in the world
pub async fn get_all_entities(_world: &World) -> EcsResult<Vec<Entity>> {
    Err(EcsError::ModuleNotFound("get_all_entities not implemented - systems/ecs required".to_string()))
}

/// Get the total count of entities
pub async fn get_entity_count(_world: &World) -> EcsResult<usize> {
    Err(EcsError::ModuleNotFound("get_entity_count not implemented - systems/ecs required".to_string()))
}

/// Get an entity by ID (creates Entity handle with current generation)
pub async fn get_entity(_world: &World, _entity_id: crate::model::entity::EntityId) -> EcsResult<Entity> {
    Err(EcsError::ModuleNotFound("get_entity not implemented - systems/ecs required".to_string()))
}

/// Get the current generation of an entity
pub async fn get_generation(_world: &World, _entity_id: crate::model::entity::EntityId) -> EcsResult<crate::model::entity::Generation> {
    Err(EcsError::ModuleNotFound("get_generation not implemented - systems/ecs required".to_string()))
}

/// Spawn entity with specific ID (useful for deserialization)
pub async fn spawn_entity_with_id(_world: &World, _entity_id: crate::model::entity::EntityId, _components: Vec<Component>) -> EcsResult<Entity> {
    Err(EcsError::ModuleNotFound("spawn_entity_with_id not implemented - systems/ecs required".to_string()))
}