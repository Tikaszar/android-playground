//! Storage contracts for the ECS

use async_trait::async_trait;
use crate::entity::EntityId;
use crate::error::EcsResult;

/// Storage type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    Dense,
    Sparse,
    Pooled,
}

/// Trait for component storage implementations
/// This is the contract that storage systems must implement
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get the storage type
    fn storage_type(&self) -> StorageType;
    
    /// Check if an entity has this component
    async fn contains(&self, entity: EntityId) -> bool;
    
    /// Clear all components
    async fn clear(&self) -> EcsResult<()>;
    
    /// Get the number of components
    async fn len(&self) -> usize;
    
    /// Check if storage is empty
    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
    
    /// Get all entities with this component
    async fn entities(&self) -> Vec<EntityId>;
    
    /// Mark an entity as dirty
    async fn mark_dirty(&self, entity: EntityId) -> EcsResult<()>;
    
    /// Get all dirty entities
    async fn get_dirty(&self) -> Vec<EntityId>;
    
    /// Clear dirty flags
    async fn clear_dirty(&self) -> EcsResult<()>;
}