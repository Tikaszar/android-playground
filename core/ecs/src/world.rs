//! World contract for the ECS
//! 
//! This defines the contract that the World implementation must fulfill.

use async_trait::async_trait;
use bytes::Bytes;
use crate::{
    ComponentData, ComponentId, EntityId, ChannelId,
    StorageType, EcsResult
};

/// The World contract - defines what a World implementation must provide
/// 
/// The World is the central container for all ECS data and functionality.
/// It manages entities, components, systems, and messaging.
#[async_trait]
pub trait WorldContract: Send + Sync {
    // Entity management
    
    /// Spawn a single entity without components
    async fn spawn_entity(&self) -> EcsResult<EntityId>;
    
    /// Despawn a batch of entities
    async fn despawn_batch(&self, entities: Vec<EntityId>) -> EcsResult<()>;
    
    // Component management
    
    /// Register a component type
    async fn register_component<T: ComponentData>(&self) -> EcsResult<()>;
    
    /// Register a component with specific storage type
    async fn register_component_with_storage<T: ComponentData>(&self, storage_type: StorageType) -> EcsResult<()>;
    
    /// Check if an entity has a component
    async fn has_component(&self, entity: EntityId, component_id: ComponentId) -> bool;
    
    /// Get a component from an entity
    async fn get_component<T: ComponentData>(&self, entity: EntityId) -> EcsResult<T>;
    
    // Query system
    
    /// Execute a query to find entities
    async fn query_entities(&self, required: Vec<ComponentId>, excluded: Vec<ComponentId>) -> EcsResult<Vec<EntityId>>;
    
    // System execution
    
    /// Update the world by executing all systems
    async fn update(&self, delta_time: f32) -> EcsResult<()>;
    
    // Messaging
    
    /// Publish a message to a channel
    async fn publish(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
    
    /// Subscribe to a channel
    async fn subscribe(&self, channel: ChannelId, handler_id: String) -> EcsResult<()>;
    
    /// Unsubscribe from a channel  
    async fn unsubscribe(&self, channel: ChannelId, handler_id: &str) -> EcsResult<()>;
    
    // Memory management
    
    /// Run garbage collection
    async fn run_gc(&self) -> EcsResult<usize>;
    
    /// Check if the world is empty
    async fn is_empty(&self) -> bool;
}