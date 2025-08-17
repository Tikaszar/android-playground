use async_trait::async_trait;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use crate::entity::EntityId;
use crate::component::{ComponentBox, ComponentId};
use crate::error::{EcsError, EcsResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    Dense,
    Sparse,
    Pooled,
}

#[async_trait]
pub trait ComponentStorage: Send + Sync {
    fn storage_type(&self) -> StorageType;
    
    async fn insert(&self, entity: EntityId, component: ComponentBox) -> EcsResult<()>;
    
    async fn insert_batch(&self, components: Vec<(EntityId, ComponentBox)>) -> EcsResult<()>;
    
    async fn remove(&self, entity: EntityId) -> EcsResult<ComponentBox>;
    
    async fn remove_batch(&self, entities: Vec<EntityId>) -> EcsResult<Vec<ComponentBox>>;
    
    async fn get_raw(&self, entity: EntityId) -> EcsResult<ComponentBox>;
    
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

pub struct SparseStorage {
    components: Arc<DashMap<EntityId, ComponentBox>>,
    dirty: Arc<DashMap<EntityId, ()>>,
    component_id: ComponentId,
}

impl SparseStorage {
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            components: Arc::new(DashMap::new()),
            dirty: Arc::new(DashMap::new()),
            component_id,
        }
    }
}

#[async_trait]
impl ComponentStorage for SparseStorage {
    fn storage_type(&self) -> StorageType {
        StorageType::Sparse
    }
    
    async fn insert(&self, entity: EntityId, component: ComponentBox) -> EcsResult<()> {
        self.components.insert(entity, component);
        self.dirty.insert(entity, ());
        Ok(())
    }
    
    async fn insert_batch(&self, components: Vec<(EntityId, ComponentBox)>) -> EcsResult<()> {
        for (entity, component) in components {
            self.components.insert(entity, component);
            self.dirty.insert(entity, ());
        }
        Ok(())
    }
    
    async fn remove(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        self.dirty.remove(&entity);
        self.components.remove(&entity)
            .map(|(_, component)| component)
            .ok_or_else(|| EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })
    }
    
    async fn remove_batch(&self, entities: Vec<EntityId>) -> EcsResult<Vec<ComponentBox>> {
        let mut results = Vec::with_capacity(entities.len());
        for entity in entities {
            self.dirty.remove(&entity);
            if let Some((_, component)) = self.components.remove(&entity) {
                results.push(component);
            }
        }
        Ok(results)
    }
    
    async fn get_raw(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        self.components.get(&entity)
            .map(|entry| {
                let bytes = futures::executor::block_on(entry.value().serialize())
                    .unwrap_or_else(|_| bytes::Bytes::new());
                Box::new(RawComponent { data: bytes }) as ComponentBox
            })
            .ok_or_else(|| EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })
    }
    
    async fn contains(&self, entity: EntityId) -> bool {
        self.components.contains_key(&entity)
    }
    
    async fn clear(&self) -> EcsResult<()> {
        self.components.clear();
        self.dirty.clear();
        Ok(())
    }
    
    async fn len(&self) -> usize {
        self.components.len()
    }
    
    async fn entities(&self) -> Vec<EntityId> {
        self.components.iter()
            .map(|entry| *entry.key())
            .collect()
    }
    
    async fn mark_dirty(&self, entity: EntityId) -> EcsResult<()> {
        if self.components.contains_key(&entity) {
            self.dirty.insert(entity, ());
            Ok(())
        } else {
            Err(EcsError::EntityNotFound(entity))
        }
    }
    
    async fn get_dirty(&self) -> Vec<EntityId> {
        self.dirty.iter()
            .map(|entry| *entry.key())
            .collect()
    }
    
    async fn clear_dirty(&self) -> EcsResult<()> {
        self.dirty.clear();
        Ok(())
    }
}

pub struct DenseStorage {
    entities: Arc<RwLock<Vec<EntityId>>>,
    components: Arc<RwLock<Vec<ComponentBox>>>,
    entity_to_index: Arc<DashMap<EntityId, usize>>,
    dirty: Arc<DashMap<EntityId, ()>>,
    component_id: ComponentId,
}

impl DenseStorage {
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            entities: Arc::new(RwLock::new(Vec::new())),
            components: Arc::new(RwLock::new(Vec::new())),
            entity_to_index: Arc::new(DashMap::new()),
            dirty: Arc::new(DashMap::new()),
            component_id,
        }
    }
}

#[async_trait]
impl ComponentStorage for DenseStorage {
    fn storage_type(&self) -> StorageType {
        StorageType::Dense
    }
    
    async fn insert(&self, entity: EntityId, component: ComponentBox) -> EcsResult<()> {
        if let Some(index) = self.entity_to_index.get(&entity).map(|e| *e.value()) {
            let mut components = self.components.write();
            if index < components.len() {
                components[index] = component;
            }
            self.dirty.insert(entity, ());
        } else {
            let mut entities = self.entities.write();
            let mut components = self.components.write();
            let index = entities.len();
            entities.push(entity);
            components.push(component);
            drop(entities);
            drop(components);
            self.entity_to_index.insert(entity, index);
            self.dirty.insert(entity, ());
        }
        Ok(())
    }
    
    async fn insert_batch(&self, batch: Vec<(EntityId, ComponentBox)>) -> EcsResult<()> {
        let mut entities = self.entities.write();
        let mut components = self.components.write();
        
        for (entity, component) in batch {
            if let Some(index) = self.entity_to_index.get(&entity).map(|e| *e.value()) {
                if index < components.len() {
                    components[index] = component;
                }
            } else {
                let index = entities.len();
                entities.push(entity);
                components.push(component);
                self.entity_to_index.insert(entity, index);
            }
            self.dirty.insert(entity, ());
        }
        
        Ok(())
    }
    
    async fn remove(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        self.dirty.remove(&entity);
        
        if let Some((_, index)) = self.entity_to_index.remove(&entity) {
            let mut entities = self.entities.write();
            let mut components = self.components.write();
            
            if index >= entities.len() {
                return Err(EcsError::ComponentNotFound {
                    entity,
                    component: format!("{:?}", self.component_id),
                });
            }
            
            let component = if index == entities.len() - 1 {
                entities.pop();
                components.pop().unwrap()
            } else {
                let last_entity = entities.pop().unwrap();
                let last_component = components.pop().unwrap();
                
                entities[index] = last_entity;
                let removed = std::mem::replace(&mut components[index], last_component);
                
                self.entity_to_index.insert(last_entity, index);
                removed
            };
            
            Ok(component)
        } else {
            Err(EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })
        }
    }
    
    async fn remove_batch(&self, entities_to_remove: Vec<EntityId>) -> EcsResult<Vec<ComponentBox>> {
        let mut removed = Vec::with_capacity(entities_to_remove.len());
        
        for entity in entities_to_remove {
            if let Ok(component) = self.remove(entity).await {
                removed.push(component);
            }
        }
        
        Ok(removed)
    }
    
    async fn get_raw(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        if let Some(index) = self.entity_to_index.get(&entity).map(|e| *e.value()) {
            let component_clone = {
                let components = self.components.read();
                if index < components.len() {
                    let bytes = futures::executor::block_on(components[index].serialize())
                        .unwrap_or_else(|_| bytes::Bytes::new());
                    Some(bytes)
                } else {
                    None
                }
            };
            
            if let Some(bytes) = component_clone {
                Ok(Box::new(RawComponent { data: bytes }) as ComponentBox)
            } else {
                Err(EcsError::ComponentNotFound {
                    entity,
                    component: format!("{:?}", self.component_id),
                })
            }
        } else {
            Err(EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })
        }
    }
    
    async fn contains(&self, entity: EntityId) -> bool {
        self.entity_to_index.contains_key(&entity)
    }
    
    async fn clear(&self) -> EcsResult<()> {
        self.entities.write().clear();
        self.components.write().clear();
        self.entity_to_index.clear();
        self.dirty.clear();
        Ok(())
    }
    
    async fn len(&self) -> usize {
        self.entities.read().len()
    }
    
    async fn entities(&self) -> Vec<EntityId> {
        self.entities.read().clone()
    }
    
    async fn mark_dirty(&self, entity: EntityId) -> EcsResult<()> {
        if self.entity_to_index.contains_key(&entity) {
            self.dirty.insert(entity, ());
            Ok(())
        } else {
            Err(EcsError::EntityNotFound(entity))
        }
    }
    
    async fn get_dirty(&self) -> Vec<EntityId> {
        self.dirty.iter()
            .map(|entry| *entry.key())
            .collect()
    }
    
    async fn clear_dirty(&self) -> EcsResult<()> {
        self.dirty.clear();
        Ok(())
    }
}

struct RawComponent {
    data: bytes::Bytes,
}

#[async_trait]
impl crate::component::Component for RawComponent {
    async fn serialize(&self) -> EcsResult<bytes::Bytes> {
        Ok(self.data.clone())
    }
    
    async fn deserialize(bytes: &bytes::Bytes) -> EcsResult<Self> {
        Ok(Self { data: bytes.clone() })
    }
}