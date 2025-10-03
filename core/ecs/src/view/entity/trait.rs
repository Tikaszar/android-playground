//! Entity view trait - API contract only

use async_trait::async_trait;
use crate::{
    EcsResult,
    model::{World, Entity, EntityId, Generation, Component},
};

/// Entity management API contract
#[async_trait]
pub trait EntityView: Send + Sync {
    /// Spawn a new entity with components
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity>;

    /// Spawn multiple entities in batch for efficiency
    async fn spawn_batch(&self, world: &World, batches: Vec<Vec<Component>>) -> EcsResult<Vec<Entity>>;

    /// Despawn an entity
    async fn despawn_entity(&self, world: &World, entity: Entity) -> EcsResult<()>;

    /// Despawn multiple entities in batch
    async fn despawn_batch(&self, world: &World, entities: Vec<Entity>) -> EcsResult<()>;

    /// Clone an entity with all its components
    async fn clone_entity(&self, world: &World, entity: Entity) -> EcsResult<Entity>;

    /// Check if an entity exists (regardless of generation)
    async fn exists(&self, world: &World, entity: Entity) -> EcsResult<bool>;

    /// Check if an entity is alive (valid generation)
    async fn is_alive(&self, world: &World, entity: Entity) -> EcsResult<bool>;

    /// Get all entities in the world
    async fn get_all_entities(&self, world: &World) -> EcsResult<Vec<Entity>>;

    /// Get the total count of entities
    async fn get_entity_count(&self, world: &World) -> EcsResult<usize>;

    /// Get an entity by ID (creates Entity handle with current generation)
    async fn get_entity(&self, world: &World, entity_id: EntityId) -> EcsResult<Entity>;

    /// Get the current generation of an entity
    async fn get_generation(&self, world: &World, entity_id: EntityId) -> EcsResult<Generation>;

    /// Spawn entity with specific ID (useful for deserialization)
    async fn spawn_entity_with_id(&self, world: &World, entity_id: EntityId, components: Vec<Component>) -> EcsResult<Entity>;
}