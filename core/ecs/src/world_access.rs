//! Static access to World functionality
//! 
//! This module provides static functions to interact with the World implementation.
//! The actual implementation (systems/ecs) will register the command sender during initialization.

use crate::{EntityId, ComponentId, EcsResult, EcsError, WorldCommand};
use bytes::Bytes;
use once_cell::sync::OnceCell;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

// Static storage for the command sender
// This will be set by systems/ecs during initialization
static COMMAND_SENDER: OnceCell<mpsc::UnboundedSender<WorldCommand>> = OnceCell::new();

/// Register the command sender (called by systems/ecs)
pub fn register_command_sender(sender: mpsc::UnboundedSender<WorldCommand>) {
    COMMAND_SENDER.set(sender).ok();
}

/// Spawn a new entity
pub async fn spawn_entity() -> EcsResult<EntityId> {
    let (tx, rx) = oneshot::channel();
    
    COMMAND_SENDER
        .get()
        .ok_or(EcsError::NotInitialized)?
        .send(WorldCommand::SpawnEntity { response: tx })
        .map_err(|_| EcsError::SendError)?;
    
    rx.await.map_err(|_| EcsError::ReceiveError)?
}

/// Spawn multiple entities
pub async fn spawn_batch(count: usize) -> EcsResult<Vec<EntityId>> {
    let (tx, rx) = oneshot::channel();
    
    COMMAND_SENDER
        .get()
        .ok_or(EcsError::NotInitialized)?
        .send(WorldCommand::SpawnBatch { count, response: tx })
        .map_err(|_| EcsError::SendError)?;
    
    rx.await.map_err(|_| EcsError::ReceiveError)?
}

/// Despawn an entity
pub async fn despawn_entity(entity: EntityId) -> EcsResult<()> {
    let (tx, rx) = oneshot::channel();
    
    COMMAND_SENDER
        .get()
        .ok_or(EcsError::NotInitialized)?
        .send(WorldCommand::DespawnEntity { entity, response: tx })
        .map_err(|_| EcsError::SendError)?;
    
    rx.await.map_err(|_| EcsError::ReceiveError)?
}

/// Despawn multiple entities
pub async fn despawn_batch(entities: Vec<EntityId>) -> EcsResult<()> {
    let (tx, rx) = oneshot::channel();
    
    COMMAND_SENDER
        .get()
        .ok_or(EcsError::NotInitialized)?
        .send(WorldCommand::DespawnBatch { entities, response: tx })
        .map_err(|_| EcsError::SendError)?;
    
    rx.await.map_err(|_| EcsError::ReceiveError)?
}

/// Query for entities with specific components
pub async fn query_entities(
    required: Vec<ComponentId>,
    excluded: Vec<ComponentId>,
) -> EcsResult<Vec<EntityId>> {
    let (tx, rx) = oneshot::channel();
    
    COMMAND_SENDER
        .get()
        .ok_or(EcsError::NotInitialized)?
        .send(WorldCommand::QueryEntities { required, excluded, response: tx })
        .map_err(|_| EcsError::SendError)?;
    
    rx.await.map_err(|_| EcsError::ReceiveError)?
}

/// Get a component from an entity (as bytes)
pub async fn get_component_bytes(
    entity: EntityId,
    component_id: ComponentId,
) -> EcsResult<Bytes> {
    let (tx, rx) = oneshot::channel();
    
    COMMAND_SENDER
        .get()
        .ok_or(EcsError::NotInitialized)?
        .send(WorldCommand::GetComponent { entity, component_id, response: tx })
        .map_err(|_| EcsError::SendError)?;
    
    rx.await.map_err(|_| EcsError::ReceiveError)?
}

/// Set a component on an entity (as bytes)
pub async fn set_component_bytes(
    entity: EntityId,
    component_id: ComponentId,
    data: Bytes,
) -> EcsResult<()> {
    let (tx, rx) = oneshot::channel();
    
    COMMAND_SENDER
        .get()
        .ok_or(EcsError::NotInitialized)?
        .send(WorldCommand::SetComponent { entity, component_id, data, response: tx })
        .map_err(|_| EcsError::SendError)?;
    
    rx.await.map_err(|_| EcsError::ReceiveError)?
}

/// Add a component to an entity
pub async fn add_component_bytes(
    entity: EntityId,
    component_id: ComponentId,
    data: Bytes,
) -> EcsResult<()> {
    let (tx, rx) = oneshot::channel();
    
    COMMAND_SENDER
        .get()
        .ok_or(EcsError::NotInitialized)?
        .send(WorldCommand::AddComponent { entity, component_id, data, response: tx })
        .map_err(|_| EcsError::SendError)?;
    
    rx.await.map_err(|_| EcsError::ReceiveError)?
}

/// Remove a component from an entity
pub async fn remove_component(
    entity: EntityId,
    component_id: ComponentId,
) -> EcsResult<()> {
    let (tx, rx) = oneshot::channel();
    
    COMMAND_SENDER
        .get()
        .ok_or(EcsError::NotInitialized)?
        .send(WorldCommand::RemoveComponent { entity, component_id, response: tx })
        .map_err(|_| EcsError::SendError)?;
    
    rx.await.map_err(|_| EcsError::ReceiveError)?
}

/// Check if an entity has a component
pub async fn has_component(
    entity: EntityId,
    component_id: ComponentId,
) -> bool {
    let (tx, rx) = oneshot::channel();
    
    if let Some(sender) = COMMAND_SENDER.get() {
        if sender.send(WorldCommand::HasComponent { entity, component_id, response: tx }).is_ok() {
            if let Ok(result) = rx.await {
                return result;
            }
        }
    }
    
    false
}

/// Check if an entity is alive
pub async fn is_entity_alive(entity: EntityId) -> bool {
    let (tx, rx) = oneshot::channel();
    
    if let Some(sender) = COMMAND_SENDER.get() {
        if sender.send(WorldCommand::IsAlive { entity, response: tx }).is_ok() {
            if let Ok(result) = rx.await {
                return result;
            }
        }
    }
    
    false
}