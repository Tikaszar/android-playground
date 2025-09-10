//! Component storage implementations for the unified ECS

use async_trait::async_trait;
use std::collections::HashMap;
use playground_core_types::{Shared, shared};
use playground_core_ecs::{EntityId, ComponentId, EcsError, EcsResult};
use crate::component::ComponentBox;

/// Storage type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    Dense,
    Sparse,
    Pooled,
}

/// Trait for component storage implementations
#[async_trait]
pub trait Storage: Send + Sync {
    fn storage_type(&self) -> StorageType;
    
    async fn insert(&self, entity: EntityId, component: ComponentBox) -> EcsResult<()>;
    
    async fn insert_batch(&self, components: Vec<(EntityId, ComponentBox)>) -> EcsResult<()>;
    
    async fn remove(&self, entity: EntityId) -> EcsResult<ComponentBox>;
    
    async fn remove_batch(&self, entities: Vec<EntityId>) -> EcsResult<Vec<ComponentBox>>;
    
    async fn get_raw(&self, entity: EntityId) -> EcsResult<ComponentBox>;
    
    async fn get_raw_mut(&self, entity: EntityId) -> EcsResult<Shared<ComponentBox>>;
    
    async fn contains(&self, entity: EntityId) -> bool;
    
    async fn clear(&self) -> EcsResult<()>;
    
    async fn len(&self) -> usize;
    
    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
    
    async fn entities(&self) -> Vec<EntityId>;
    
    async fn mark_dirty(&self, entity: EntityId) -> EcsResult<()>;
    
    async fn get_dirty(&self) -> Vec<EntityId>;
    
    async fn clear_dirty(&self) -> EcsResult<()>;
}

/// Sparse storage implementation using HashMap
pub struct SparseStorage {
    components: Shared<HashMap<EntityId, Shared<ComponentBox>>>,
    dirty: Shared<HashMap<EntityId, ()>>,
    component_id: ComponentId,
}

impl SparseStorage {
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            components: shared(HashMap::new()),
            dirty: shared(HashMap::new()),
            component_id,
        }
    }
}

#[async_trait]
impl Storage for SparseStorage {
    fn storage_type(&self) -> StorageType {
        StorageType::Sparse
    }
    
    async fn insert(&self, entity: EntityId, component: ComponentBox) -> EcsResult<()> {
        self.components.write().await.insert(entity, shared(component));
        self.dirty.write().await.insert(entity, ());
        Ok(())
    }
    
    async fn insert_batch(&self, components: Vec<(EntityId, ComponentBox)>) -> EcsResult<()> {
        let mut storage = self.components.write().await;
        let mut dirty = self.dirty.write().await;
        
        for (entity, component) in components {
            storage.insert(entity, shared(component));
            dirty.insert(entity, ());
        }
        Ok(())
    }
    
    async fn remove(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        self.dirty.write().await.remove(&entity);
        let shared_comp = self.components.write().await.remove(&entity)
            .ok_or_else(|| EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })?;
        
        // Try to extract the component from Shared
        // This will fail if there are other references
        match std::sync::Arc::try_unwrap(shared_comp) {
            Ok(rwlock) => {
                let component = rwlock.into_inner();
                Ok(component)
            }
            Err(shared_comp) => {
                // If we can't unwrap, clone the component
                let guard = shared_comp.read().await;
                let cloned = guard.clone();
                Ok(cloned)
            }
        }
    }
    
    async fn remove_batch(&self, entities: Vec<EntityId>) -> EcsResult<Vec<ComponentBox>> {
        let mut results = Vec::with_capacity(entities.len());
        for entity in entities {
            results.push(self.remove(entity).await?);
        }
        Ok(results)
    }
    
    async fn get_raw(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        let components = self.components.read().await;
        let shared_comp = components.get(&entity)
            .ok_or_else(|| EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })?;
        
        let guard = shared_comp.read().await;
        Ok(guard.clone())
    }
    
    async fn get_raw_mut(&self, entity: EntityId) -> EcsResult<Shared<ComponentBox>> {
        let components = self.components.read().await;
        components.get(&entity)
            .cloned()
            .ok_or_else(|| EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })
    }
    
    async fn contains(&self, entity: EntityId) -> bool {
        self.components.read().await.contains_key(&entity)
    }
    
    async fn clear(&self) -> EcsResult<()> {
        self.components.write().await.clear();
        self.dirty.write().await.clear();
        Ok(())
    }
    
    async fn len(&self) -> usize {
        self.components.read().await.len()
    }
    
    async fn entities(&self) -> Vec<EntityId> {
        self.components.read().await.keys().cloned().collect()
    }
    
    async fn mark_dirty(&self, entity: EntityId) -> EcsResult<()> {
        if self.components.read().await.contains_key(&entity) {
            self.dirty.write().await.insert(entity, ());
            Ok(())
        } else {
            Err(EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })
        }
    }
    
    async fn get_dirty(&self) -> Vec<EntityId> {
        self.dirty.read().await.keys().cloned().collect()
    }
    
    async fn clear_dirty(&self) -> EcsResult<()> {
        self.dirty.write().await.clear();
        Ok(())
    }
}

/// Dense storage implementation using Vec with entity index mapping
pub struct DenseStorage {
    entities: Shared<Vec<EntityId>>,
    entity_to_index: Shared<HashMap<EntityId, usize>>,
    components: Shared<Vec<ComponentBox>>,
    dirty: Shared<HashMap<EntityId, ()>>,
    component_id: ComponentId,
}

impl DenseStorage {
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            entities: shared(Vec::new()),
            entity_to_index: shared(HashMap::new()),
            components: shared(Vec::new()),
            dirty: shared(HashMap::new()),
            component_id,
        }
    }
}

#[async_trait]
impl Storage for DenseStorage {
    fn storage_type(&self) -> StorageType {
        StorageType::Dense
    }
    
    async fn insert(&self, entity: EntityId, component: ComponentBox) -> EcsResult<()> {
        let mut entities = self.entities.write().await;
        let mut entity_to_index = self.entity_to_index.write().await;
        let mut components = self.components.write().await;
        
        if let Some(&index) = entity_to_index.get(&entity) {
            // Update existing
            components[index] = component;
        } else {
            // Add new
            let index = entities.len();
            entities.push(entity);
            entity_to_index.insert(entity, index);
            components.push(component);
        }
        
        self.dirty.write().await.insert(entity, ());
        Ok(())
    }
    
    async fn insert_batch(&self, batch: Vec<(EntityId, ComponentBox)>) -> EcsResult<()> {
        for (entity, component) in batch {
            self.insert(entity, component).await?;
        }
        Ok(())
    }
    
    async fn remove(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        let mut entities = self.entities.write().await;
        let mut entity_to_index = self.entity_to_index.write().await;
        let mut components = self.components.write().await;
        
        let index = entity_to_index.remove(&entity)
            .ok_or_else(|| EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })?;
        
        // Swap remove to maintain density
        let component = if index == entities.len() - 1 {
            // Last element, just pop
            entities.pop();
            components.pop().unwrap()
        } else {
            // Swap with last and update mapping
            let last_entity = entities[entities.len() - 1];
            entities.swap_remove(index);
            entity_to_index.insert(last_entity, index);
            components.swap_remove(index)
        };
        
        self.dirty.write().await.remove(&entity);
        Ok(component)
    }
    
    async fn remove_batch(&self, entities: Vec<EntityId>) -> EcsResult<Vec<ComponentBox>> {
        let mut results = Vec::with_capacity(entities.len());
        for entity in entities {
            results.push(self.remove(entity).await?);
        }
        Ok(results)
    }
    
    async fn get_raw(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        let entity_to_index = self.entity_to_index.read().await;
        let components = self.components.read().await;
        
        let &index = entity_to_index.get(&entity)
            .ok_or_else(|| EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })?;
        
        Ok(components[index].clone())
    }
    
    async fn get_raw_mut(&self, entity: EntityId) -> EcsResult<Shared<ComponentBox>> {
        let entity_to_index = self.entity_to_index.read().await;
        
        let &index = entity_to_index.get(&entity)
            .ok_or_else(|| EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })?;
        
        // For dense storage, we return a shared reference to the component
        // This is a bit hacky but avoids dyn trait objects
        Ok(shared(self.components.read().await[index].clone()))
    }
    
    async fn contains(&self, entity: EntityId) -> bool {
        self.entity_to_index.read().await.contains_key(&entity)
    }
    
    async fn clear(&self) -> EcsResult<()> {
        self.entities.write().await.clear();
        self.entity_to_index.write().await.clear();
        self.components.write().await.clear();
        self.dirty.write().await.clear();
        Ok(())
    }
    
    async fn len(&self) -> usize {
        self.entities.read().await.len()
    }
    
    async fn entities(&self) -> Vec<EntityId> {
        self.entities.read().await.clone()
    }
    
    async fn mark_dirty(&self, entity: EntityId) -> EcsResult<()> {
        if self.entity_to_index.read().await.contains_key(&entity) {
            self.dirty.write().await.insert(entity, ());
            Ok(())
        } else {
            Err(EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })
        }
    }
    
    async fn get_dirty(&self) -> Vec<EntityId> {
        self.dirty.read().await.keys().cloned().collect()
    }
    
    async fn clear_dirty(&self) -> EcsResult<()> {
        self.dirty.write().await.clear();
        Ok(())
    }
}

/// Component storage wrapper that abstracts over storage types
/// Uses enum pattern to avoid dyn trait objects
pub enum ComponentStorage {
    Sparse(SparseStorage),
    Dense(DenseStorage),
}

impl ComponentStorage {
    /// Create a new sparse storage
    pub fn new_sparse(component_id: ComponentId) -> Self {
        Self::Sparse(SparseStorage::new(component_id))
    }
    
    /// Create a new dense storage
    pub fn new_dense(component_id: ComponentId) -> Self {
        Self::Dense(DenseStorage::new(component_id))
    }
    
    // Delegate all methods to the underlying storage
    pub async fn insert(&self, entity: EntityId, component: ComponentBox) -> EcsResult<()> {
        match self {
            Self::Sparse(storage) => storage.insert(entity, component).await,
            Self::Dense(storage) => storage.insert(entity, component).await,
        }
    }
    
    pub async fn remove(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        match self {
            Self::Sparse(storage) => storage.remove(entity).await,
            Self::Dense(storage) => storage.remove(entity).await,
        }
    }
    
    pub async fn get_raw(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        match self {
            Self::Sparse(storage) => storage.get_raw(entity).await,
            Self::Dense(storage) => storage.get_raw(entity).await,
        }
    }
    
    pub async fn contains(&self, entity: EntityId) -> bool {
        match self {
            Self::Sparse(storage) => storage.contains(entity).await,
            Self::Dense(storage) => storage.contains(entity).await,
        }
    }
    
    pub async fn get_dirty(&self) -> Vec<EntityId> {
        match self {
            Self::Sparse(storage) => storage.get_dirty().await,
            Self::Dense(storage) => storage.get_dirty().await,
        }
    }
    
    pub async fn clear_dirty(&self) -> EcsResult<()> {
        match self {
            Self::Sparse(storage) => storage.clear_dirty().await,
            Self::Dense(storage) => storage.clear_dirty().await,
        }
    }
}