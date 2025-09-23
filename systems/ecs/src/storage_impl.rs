//! Storage implementation - all the logic for component storage
//! 
//! This implements all operations on the ComponentStorage data structure from core/ecs.

use playground_core_ecs::{
    ComponentStorage, StorageType, Component, ComponentId, EntityId, Generation,
    CoreResult, CoreError, component_not_found
};

/// Implementation of all ComponentStorage operations
pub struct StorageImpl;

impl StorageImpl {
    /// Insert a component for an entity
    pub async fn insert(storage: &ComponentStorage, entity: EntityId, component: Component) -> CoreResult<()> {
        match storage.storage_type {
            StorageType::Dense => {
                let mut dense_storage = storage.dense.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Dense storage not initialized".into()))?
                    .write().await;
                
                let index = entity.index() as usize;
                
                // Grow storage if needed
                if index >= dense_storage.len() {
                    dense_storage.resize(index + 1, None);
                }
                
                dense_storage[index] = Some(component);
                Ok(())
            }
            StorageType::Sparse => {
                let mut sparse_storage = storage.sparse.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Sparse storage not initialized".into()))?
                    .write().await;
                
                sparse_storage.insert(entity, component);
                Ok(())
            }
        }
    }
    
    /// Get a component for an entity
    pub async fn get(storage: &ComponentStorage, entity: EntityId) -> CoreResult<Component> {
        match storage.storage_type {
            StorageType::Dense => {
                let dense_storage = storage.dense.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Dense storage not initialized".into()))?
                    .read().await;
                
                let index = entity.index() as usize;
                
                dense_storage.get(index)
                    .and_then(|opt| opt.as_ref())
                    .cloned()
                    .ok_or_else(|| component_not_found(entity, ComponentId(0)))
            }
            StorageType::Sparse => {
                let sparse_storage = storage.sparse.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Sparse storage not initialized".into()))?
                    .read().await;
                
                sparse_storage.get(&entity)
                    .cloned()
                    .ok_or_else(|| component_not_found(entity, ComponentId(0)))
            }
        }
    }
    
    /// Remove a component for an entity
    pub async fn remove(storage: &ComponentStorage, entity: EntityId) -> CoreResult<()> {
        match storage.storage_type {
            StorageType::Dense => {
                let mut dense_storage = storage.dense.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Dense storage not initialized".into()))?
                    .write().await;
                
                let index = entity.index() as usize;
                
                if index < dense_storage.len() {
                    dense_storage[index] = None;
                }
                Ok(())
            }
            StorageType::Sparse => {
                let mut sparse_storage = storage.sparse.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Sparse storage not initialized".into()))?
                    .write().await;
                
                sparse_storage.remove(&entity);
                Ok(())
            }
        }
    }
    
    /// Check if entity has a component
    pub async fn contains(storage: &ComponentStorage, entity: EntityId) -> bool {
        match storage.storage_type {
            StorageType::Dense => {
                if let Some(dense_storage) = &storage.dense {
                    let storage_guard = dense_storage.read().await;
                    let index = entity.index() as usize;
                    index < storage_guard.len() && storage_guard[index].is_some()
                } else {
                    false
                }
            }
            StorageType::Sparse => {
                if let Some(sparse_storage) = &storage.sparse {
                    let storage_guard = sparse_storage.read().await;
                    storage_guard.contains_key(&entity)
                } else {
                    false
                }
            }
        }
    }
    
    /// Get all entities with this component
    pub async fn entities(storage: &ComponentStorage) -> Vec<EntityId> {
        match storage.storage_type {
            StorageType::Dense => {
                if let Some(dense_storage) = &storage.dense {
                    let storage_guard = dense_storage.read().await;
                    storage_guard.iter()
                        .enumerate()
                        .filter_map(|(index, opt)| {
                            opt.as_ref().map(|_| EntityId::new(index as u32))
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }
            StorageType::Sparse => {
                if let Some(sparse_storage) = &storage.sparse {
                    let storage_guard = sparse_storage.read().await;
                    storage_guard.keys().copied().collect()
                } else {
                    Vec::new()
                }
            }
        }
    }
    
    /// Clear all components
    pub async fn clear(storage: &ComponentStorage) -> CoreResult<()> {
        match storage.storage_type {
            StorageType::Dense => {
                if let Some(dense_storage) = &storage.dense {
                    let mut storage_guard = dense_storage.write().await;
                    storage_guard.clear();
                }
                Ok(())
            }
            StorageType::Sparse => {
                if let Some(sparse_storage) = &storage.sparse {
                    let mut storage_guard = sparse_storage.write().await;
                    storage_guard.clear();
                }
                Ok(())
            }
        }
    }
}