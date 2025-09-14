//! World command types for communicating with the ECS World
//! 
//! This module defines commands that can be sent to the World implementation
//! through the messaging system, allowing systems to interact with the ECS
//! without directly depending on systems/ecs.

use crate::{EntityId, ComponentId, EcsResult, EcsError};
use bytes::Bytes;
use tokio::sync::oneshot;
use async_trait::async_trait;

/// Commands that can be sent to the World implementation
#[derive(Debug)]
pub enum WorldCommand {
    /// Spawn a new entity
    SpawnEntity {
        response: oneshot::Sender<EcsResult<EntityId>>,
    },
    
    /// Spawn multiple entities
    SpawnBatch {
        count: usize,
        response: oneshot::Sender<EcsResult<Vec<EntityId>>>,
    },
    
    /// Despawn an entity
    DespawnEntity {
        entity: EntityId,
        response: oneshot::Sender<EcsResult<()>>,
    },
    
    /// Despawn multiple entities
    DespawnBatch {
        entities: Vec<EntityId>,
        response: oneshot::Sender<EcsResult<()>>,
    },
    
    /// Query for entities with specific components
    QueryEntities {
        required: Vec<ComponentId>,
        excluded: Vec<ComponentId>,
        response: oneshot::Sender<EcsResult<Vec<EntityId>>>,
    },
    
    /// Get a component from an entity (as bytes)
    GetComponent {
        entity: EntityId,
        component_id: ComponentId,
        response: oneshot::Sender<EcsResult<Bytes>>,
    },
    
    /// Set a component on an entity (as bytes)
    SetComponent {
        entity: EntityId,
        component_id: ComponentId,
        data: Bytes,
        response: oneshot::Sender<EcsResult<()>>,
    },
    
    /// Add a component to an entity
    AddComponent {
        entity: EntityId,
        component_id: ComponentId,
        data: Bytes,
        response: oneshot::Sender<EcsResult<()>>,
    },
    
    /// Remove a component from an entity
    RemoveComponent {
        entity: EntityId,
        component_id: ComponentId,
        response: oneshot::Sender<EcsResult<()>>,
    },
    
    /// Check if an entity has a component
    HasComponent {
        entity: EntityId,
        component_id: ComponentId,
        response: oneshot::Sender<bool>,
    },
    
    /// Check if an entity is alive
    IsAlive {
        entity: EntityId,
        response: oneshot::Sender<bool>,
    },
}

/// Trait for handling world commands
/// 
/// This will be implemented by systems/ecs World
#[async_trait]
pub trait WorldCommandHandler: Send + Sync {
    /// Handle a world command
    async fn handle_command(&self, command: WorldCommand);
}