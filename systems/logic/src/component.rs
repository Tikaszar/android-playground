use crate::error::{LogicResult, LogicError};
use bytes::{Bytes, BytesMut};
use playground_core_types::{Shared, shared, Handle};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use fnv::{FnvHashMap, FnvHashSet};

pub type ComponentId = TypeId;

// Component is now a concrete base class that all components work through
pub struct Component {
    data: Bytes,
    component_id: ComponentId,
    component_name: String,
    size_hint: usize,
}

impl Component {
    pub async fn new<T: ComponentData>(component: T) -> LogicResult<Self> {
        let data = component.serialize().await?;
        Ok(Self {
            data,
            component_id: T::component_id(),
            component_name: T::component_name().to_string(),
            size_hint: std::mem::size_of::<T>(),
        })
    }
    
    pub fn from_bytes(data: Bytes, component_id: ComponentId, component_name: String, size_hint: usize) -> Self {
        Self {
            data,
            component_id,
            component_name,
            size_hint,
        }
    }
    
    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }
    
    pub fn component_name(&self) -> &str {
        &self.component_name
    }
    
    pub fn serialize(&self) -> Bytes {
        self.data.clone()
    }
    
    pub async fn deserialize<T: ComponentData>(&self) -> LogicResult<T> {
        T::deserialize(&self.data).await
    }
    
    pub fn size_hint(&self) -> usize {
        self.size_hint
    }
}

// Trait for actual component data types
#[async_trait::async_trait]
pub trait ComponentData: Send + Sync + 'static {
    fn component_id() -> ComponentId where Self: Sized {
        TypeId::of::<Self>()
    }
    
    fn component_name() -> &'static str where Self: Sized {
        std::any::type_name::<Self>()
    }
    
    async fn serialize(&self) -> LogicResult<Bytes>;
    
    async fn deserialize(bytes: &Bytes) -> LogicResult<Self> where Self: Sized;
}

/// Networked component trait for automatic replication
#[async_trait::async_trait]
pub trait NetworkedComponent: ComponentData + Serialize + for<'de> Deserialize<'de> {
    /// Serialize component to bytes for network transmission
    async fn serialize_networked(&self) -> LogicResult<Bytes> {
        let mut buf = BytesMut::new();
        let json = serde_json::to_vec(self)
            .map_err(|e| LogicError::SerializationError(e.to_string()))?;
        buf.extend_from_slice(&json);
        Ok(buf.freeze())
    }
    
    /// Deserialize component from network bytes
    async fn deserialize_networked(bytes: &[u8]) -> LogicResult<Self> where Self: Sized {
        serde_json::from_slice(bytes)
            .map_err(|e| LogicError::SerializationError(e.to_string()))
    }
    
    /// Get replication priority (higher = more important)
    fn replication_priority(&self) -> u8 {
        128 // Default medium priority
    }
    
    /// Check if component has changed and needs replication
    fn is_dirty(&self) -> bool {
        true // Default to always dirty, override for optimization
    }
}

/// Component metadata for runtime registration
#[derive(Clone)]
pub struct ComponentInfo {
    pub type_id: TypeId,
    pub type_name: String,
    pub size: usize,
    pub networked: bool,
}

/// Component registration builder
pub struct ComponentRegistration {
    info: ComponentInfo,
}

impl ComponentRegistration {
    pub fn new<T: ComponentData>() -> Self {
        Self {
            info: ComponentInfo {
                type_id: TypeId::of::<T>(),
                type_name: T::component_name().to_string(),
                size: std::mem::size_of::<T>(),
                networked: false,
            },
        }
    }
    
    pub fn networked(mut self) -> Self {
        self.info.networked = true;
        self
    }
    
    pub fn build(self) -> ComponentInfo {
        self.info
    }
}

/// Component registry for runtime type management
pub struct ComponentRegistry {
    components: Shared<HashMap<TypeId, ComponentInfo>>,
    by_name: Shared<HashMap<String, TypeId>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: shared(HashMap::new()),
            by_name: shared(HashMap::new()),
        }
    }
    
    pub async fn register(&self, info: ComponentInfo) -> LogicResult<()> {
        let mut components = self.components.write().await;
        let mut by_name = self.by_name.write().await;
        
        components.insert(info.type_id, info.clone());
        by_name.insert(info.type_name.clone(), info.type_id);
        
        Ok(())
    }
    
    pub async fn get(&self, type_id: TypeId) -> Option<ComponentInfo> {
        self.components.read().await.get(&type_id).cloned()
    }
    
    pub async fn get_by_name(&self, name: &str) -> Option<ComponentInfo> {
        let by_name = self.by_name.read().await;
        if let Some(type_id) = by_name.get(name) {
            self.components.read().await.get(type_id).cloned()
        } else {
            None
        }
    }
    
    pub async fn is_networked(&self, type_id: TypeId) -> bool {
        self.components
            .read().await
            .get(&type_id)
            .map(|info| info.networked)
            .unwrap_or(false)
    }
}

/// Dirty tracking for networked components
pub struct DirtyTracker {
    dirty_entities: Shared<FnvHashSet<crate::entity::Entity>>,
    dirty_components: Shared<FnvHashMap<crate::entity::Entity, FnvHashSet<TypeId>>>,
}

impl DirtyTracker {
    pub fn new() -> Self {
        Self {
            dirty_entities: shared(FnvHashSet::default()),
            dirty_components: shared(FnvHashMap::default()),
        }
    }
    
    pub async fn mark_dirty(&self, entity: crate::entity::Entity, component_type: TypeId) {
        self.dirty_entities.write().await.insert(entity);
        self.dirty_components
            .write().await
            .entry(entity)
            .or_insert_with(FnvHashSet::default)
            .insert(component_type);
    }
    
    pub async fn get_dirty_batch(&self, max_count: usize) -> Vec<(crate::entity::Entity, Vec<TypeId>)> {
        let mut dirty_entities = self.dirty_entities.write().await;
        let mut dirty_components = self.dirty_components.write().await;
        
        let mut batch = Vec::new();
        let entities: Vec<_> = dirty_entities.iter().take(max_count).copied().collect();
        
        for entity in entities {
            if let Some(components) = dirty_components.remove(&entity) {
                batch.push((entity, components.into_iter().collect()));
                dirty_entities.remove(&entity);
            }
        }
        
        batch
    }
    
    pub async fn clear(&self) {
        self.dirty_entities.write().await.clear();
        self.dirty_components.write().await.clear();
    }
}