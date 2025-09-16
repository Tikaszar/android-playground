//! Storage system for components
//! 
//! Provides different storage strategies using concrete types.

use std::collections::HashMap;
use playground_core_types::{Shared, shared};
use crate::{Component, ComponentId, EntityId, CoreResult, CoreError};

/// Storage type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// Dense storage - good for common components
    Dense,
    /// Sparse storage - good for rare components
    Sparse,
}

/// Concrete component storage
pub struct ComponentStorage {
    /// Storage type
    pub storage_type: StorageType,
    
    /// Dense storage: entity index -> component
    dense: Option<Shared<Vec<Option<Component>>>>,
    
    /// Sparse storage: entity -> component
    sparse: Option<Shared<HashMap<EntityId, Component>>>,
}

impl ComponentStorage {
    /// Create new storage of the specified type
    pub fn new(storage_type: StorageType) -> Self {
        match storage_type {
            StorageType::Dense => Self {
                storage_type,
                dense: Some(shared(Vec::new())),
                sparse: None,
            },
            StorageType::Sparse => Self {
                storage_type,
                dense: None,
                sparse: Some(shared(HashMap::new())),
            },
        }
    }
    
    /// Insert a component for an entity
    pub async fn insert(&self, entity: EntityId, component: Component) -> CoreResult<()> {
        match self.storage_type {
            StorageType::Dense => {
                let mut storage = self.dense.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Dense storage not initialized".into()))?
                    .write().await;
                
                let index = entity.id() as usize;
                
                // Grow storage if needed
                if index >= storage.len() {
                    storage.resize(index + 1, None);
                }
                
                storage[index] = Some(component);
                Ok(())
            }
            StorageType::Sparse => {
                let mut storage = self.sparse.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Sparse storage not initialized".into()))?
                    .write().await;
                
                storage.insert(entity, component);
                Ok(())
            }
        }
    }
    
    /// Get a component for an entity
    pub async fn get(&self, entity: EntityId) -> CoreResult<Component> {
        match self.storage_type {
            StorageType::Dense => {
                let storage = self.dense.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Dense storage not initialized".into()))?
                    .read().await;
                
                let index = entity.id() as usize;
                
                storage.get(index)
                    .and_then(|opt| opt.as_ref())
                    .cloned()
                    .ok_or_else(|| CoreError::ComponentNotFound(entity, ComponentId(0)))
            }
            StorageType::Sparse => {
                let storage = self.sparse.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Sparse storage not initialized".into()))?
                    .read().await;
                
                storage.get(&entity)
                    .cloned()
                    .ok_or_else(|| CoreError::ComponentNotFound(entity, ComponentId(0)))
            }
        }
    }
    
    /// Remove a component for an entity
    pub async fn remove(&self, entity: EntityId) -> CoreResult<()> {
        match self.storage_type {
            StorageType::Dense => {
                let mut storage = self.dense.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Dense storage not initialized".into()))?
                    .write().await;
                
                let index = entity.id() as usize;
                
                if index < storage.len() {
                    storage[index] = None;
                }
                Ok(())
            }
            StorageType::Sparse => {
                let mut storage = self.sparse.as_ref()
                    .ok_or_else(|| CoreError::StorageError("Sparse storage not initialized".into()))?
                    .write().await;
                
                storage.remove(&entity);
                Ok(())
            }
        }
    }
    
    /// Check if entity has a component
    pub async fn contains(&self, entity: EntityId) -> bool {
        match self.storage_type {
            StorageType::Dense => {
                if let Some(storage) = &self.dense {
                    let storage = storage.read().await;
                    let index = entity.id() as usize;
                    index < storage.len() && storage[index].is_some()
                } else {
                    false
                }
            }
            StorageType::Sparse => {
                if let Some(storage) = &self.sparse {
                    let storage = storage.read().await;
                    storage.contains_key(&entity)
                } else {
                    false
                }
            }
        }
    }
    
    /// Get all entities with this component
    pub async fn entities(&self) -> Vec<EntityId> {
        match self.storage_type {
            StorageType::Dense => {
                if let Some(storage) = &self.dense {
                    let storage = storage.read().await;
                    storage.iter()
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
                if let Some(storage) = &self.sparse {
                    let storage = storage.read().await;
                    storage.keys().copied().collect()
                } else {
                    Vec::new()
                }
            }
        }
    }
    
    /// Clear all components
    pub async fn clear(&self) -> CoreResult<()> {
        match self.storage_type {
            StorageType::Dense => {
                if let Some(storage) = &self.dense {
                    let mut storage = storage.write().await;
                    storage.clear();
                }
                Ok(())
            }
            StorageType::Sparse => {
                if let Some(storage) = &self.sparse {
                    let mut storage = storage.write().await;
                    storage.clear();
                }
                Ok(())
            }
        }
    }
}