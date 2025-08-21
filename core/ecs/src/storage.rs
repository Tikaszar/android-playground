use async_trait::async_trait;
use std::collections::HashMap;
use playground_core_types::{Shared, shared};
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
    components: Shared<HashMap<EntityId, ComponentBox>>,
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
impl ComponentStorage for SparseStorage {
    fn storage_type(&self) -> StorageType {
        StorageType::Sparse
    }
    
    async fn insert(&self, entity: EntityId, component: ComponentBox) -> EcsResult<()> {
        self.components.write().await.insert(entity, component);
        self.dirty.write().await.insert(entity, ());
        Ok(())
    }
    
    async fn insert_batch(&self, components: Vec<(EntityId, ComponentBox)>) -> EcsResult<()> {
        for (entity, component) in components {
            self.components.write().await.insert(entity, component);
            self.dirty.write().await.insert(entity, ());
        }
        Ok(())
    }
    
    async fn remove(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        self.dirty.write().await.remove(&entity);
        self.components.write().await.remove(&entity)
            .ok_or_else(|| EcsError::ComponentNotFound {
                entity,
                component: format!("{:?}", self.component_id),
            })
    }
    
    async fn remove_batch(&self, entities: Vec<EntityId>) -> EcsResult<Vec<ComponentBox>> {
        let mut results = Vec::with_capacity(entities.len());
        for entity in entities {
            self.dirty.write().await.remove(&entity);
            if let Some(component) = self.components.write().await.remove(&entity) {
                results.push(component);
            }
        }
        Ok(results)
    }
    
    async fn get_raw(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        self.components.read().await.get(&entity)
            .map(|component| {
                let bytes = futures::executor::block_on(component.serialize())
                    .unwrap_or_else(|_| bytes::Bytes::new());
                Box::new(RawComponent { data: bytes }) as ComponentBox
            })
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
        self.components.read().await.keys()
            .copied()
            .collect()
    }
    
    async fn mark_dirty(&self, entity: EntityId) -> EcsResult<()> {
        if self.components.read().await.contains_key(&entity) {
            self.dirty.write().await.insert(entity, ());
            Ok(())
        } else {
            Err(EcsError::EntityNotFound(entity))
        }
    }
    
    async fn get_dirty(&self) -> Vec<EntityId> {
        self.dirty.read().await.keys()
            .copied()
            .collect()
    }
    
    async fn clear_dirty(&self) -> EcsResult<()> {
        self.dirty.write().await.clear();
        Ok(())
    }
}

pub struct DenseStorage {
    entities: Shared<Vec<EntityId>>,
    components: Shared<Vec<ComponentBox>>,
    entity_to_index: Shared<HashMap<EntityId, usize>>,
    dirty: Shared<HashMap<EntityId, ()>>,
    component_id: ComponentId,
}

impl DenseStorage {
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            entities: shared(Vec::new()),
            components: shared(Vec::new()),
            entity_to_index: shared(HashMap::new()),
            dirty: shared(HashMap::new()),
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
        if let Some(index) = self.entity_to_index.read().await.get(&entity).copied() {
            let mut components = self.components.write().await;
            if index < components.len() {
                components[index] = component;
            }
            self.dirty.write().await.insert(entity, ());
        } else {
            let mut entities = self.entities.write().await;
            let mut components = self.components.write().await;
            let index = entities.len();
            entities.push(entity);
            components.push(component);
            drop(entities);
            drop(components);
            self.entity_to_index.write().await.insert(entity, index);
            self.dirty.write().await.insert(entity, ());
        }
        Ok(())
    }
    
    async fn insert_batch(&self, batch: Vec<(EntityId, ComponentBox)>) -> EcsResult<()> {
        let mut entities = self.entities.write().await;
        let mut components = self.components.write().await;
        
        for (entity, component) in batch {
            if let Some(index) = self.entity_to_index.read().await.get(&entity).copied() {
                if index < components.len() {
                    components[index] = component;
                }
            } else {
                let index = entities.len();
                entities.push(entity);
                components.push(component);
                self.entity_to_index.write().await.insert(entity, index);
            }
            self.dirty.write().await.insert(entity, ());
        }
        
        Ok(())
    }
    
    async fn remove(&self, entity: EntityId) -> EcsResult<ComponentBox> {
        self.dirty.write().await.remove(&entity);
        
        if let Some(index) = self.entity_to_index.write().await.remove(&entity) {
            let mut entities = self.entities.write().await;
            let mut components = self.components.write().await;
            
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
                
                self.entity_to_index.write().await.insert(last_entity, index);
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
        if let Some(index) = self.entity_to_index.read().await.get(&entity).copied() {
            let component_clone = {
                let components = self.components.read().await;
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
        self.entity_to_index.read().await.contains_key(&entity)
    }
    
    async fn clear(&self) -> EcsResult<()> {
        self.entities.write().await.clear();
        self.components.write().await.clear();
        self.entity_to_index.write().await.clear();
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
            Err(EcsError::EntityNotFound(entity))
        }
    }
    
    async fn get_dirty(&self) -> Vec<EntityId> {
        self.dirty.read().await.keys()
            .copied()
            .collect()
    }
    
    async fn clear_dirty(&self) -> EcsResult<()> {
        self.dirty.write().await.clear();
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