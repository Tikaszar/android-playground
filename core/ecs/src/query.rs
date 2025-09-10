//! Query contracts for the ECS
//! 
//! This defines the contract for querying entities and components.

use async_trait::async_trait;
use crate::{ComponentId, EntityId, EcsResult};

/// Query trait for finding entities with specific components
/// 
/// Queries are the primary way to find entities that match certain
/// component criteria in the ECS.
#[async_trait]
pub trait Query: Send + Sync {
    /// Add a required component to the query
    fn with_component(self, component_id: ComponentId) -> Self where Self: Sized;
    
    /// Add an excluded component to the query
    fn without_component(self, component_id: ComponentId) -> Self where Self: Sized;
    
    /// Add an optional component to the query
    fn optional_component(self, component_id: ComponentId) -> Self where Self: Sized;
    
    /// Execute the query and return matching entities
    async fn execute(&self) -> EcsResult<Vec<EntityId>>;
    
    /// Check if an entity matches this query
    async fn matches(&self, entity: EntityId) -> bool;
    
    /// Get the count of matching entities
    async fn count(&self) -> usize {
        self.execute().await.map(|e| e.len()).unwrap_or(0)
    }
}