//! Component pool for native type storage

use std::collections::HashMap;
use playground_core_types::{Shared, shared};
use crate::model::entity::EntityId;

/// A pool that stores components of a specific type
/// This is the actual storage for components, owned by Systems
/// Each pool stores native Rust types directly, no serialization
pub struct ComponentPool<T> {
    /// Components stored by entity ID
    /// Direct native storage - no Bytes, no serialization
    pub components: Shared<HashMap<EntityId, T>>,
}

impl<T> ComponentPool<T> {
    /// Create a new empty component pool
    pub fn new() -> Self {
        Self {
            components: shared(HashMap::new()),
        }
    }

    /// Insert a component for an entity
    pub async fn insert(&self, entity: EntityId, component: T) {
        let mut components = self.components.write().await;
        components.insert(entity, component);
    }

    /// Get a component for an entity (requires async for read lock)
    pub async fn get(&self, entity: EntityId) -> Option<T>
    where
        T: Clone
    {
        let components = self.components.read().await;
        components.get(&entity).cloned()
    }

    /// Remove a component for an entity
    pub async fn remove(&self, entity: EntityId) -> Option<T> {
        let mut components = self.components.write().await;
        components.remove(&entity)
    }

    /// Check if an entity has this component
    pub async fn contains(&self, entity: EntityId) -> bool {
        let components = self.components.read().await;
        components.contains_key(&entity)
    }

    /// Get the number of components in this pool
    pub async fn len(&self) -> usize {
        let components = self.components.read().await;
        components.len()
    }

    /// Check if the pool is empty
    pub async fn is_empty(&self) -> bool {
        let components = self.components.read().await;
        components.is_empty()
    }

    /// Clear all components from the pool
    pub async fn clear(&self) {
        let mut components = self.components.write().await;
        components.clear();
    }
}

impl<T> Clone for ComponentPool<T> {
    fn clone(&self) -> Self {
        Self {
            components: self.components.clone(),
        }
    }
}