use crate::error::LogicResult;
use bytes::{Bytes, BytesMut};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

/// Component trait that all game components must implement
pub trait Component: Send + Sync + 'static {
    fn type_name() -> &'static str where Self: Sized;
}

/// Networked component trait for automatic replication
pub trait NetworkedComponent: Component + Serialize + for<'de> Deserialize<'de> {
    /// Serialize component to bytes for network transmission
    fn serialize_networked(&self) -> LogicResult<Bytes> {
        let mut buf = BytesMut::new();
        let json = serde_json::to_vec(self)
            .map_err(|e| crate::error::LogicError::SerializationError(e.to_string()))?;
        buf.extend_from_slice(&json);
        Ok(buf.freeze())
    }
    
    /// Deserialize component from network bytes
    fn deserialize_networked(bytes: &[u8]) -> LogicResult<Self> where Self: Sized {
        serde_json::from_slice(bytes)
            .map_err(|e| crate::error::LogicError::SerializationError(e.to_string()))
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
    pub migration_fn: Option<Arc<dyn Fn(&[u8]) -> LogicResult<Vec<u8>> + Send + Sync>>,
}

/// Component registration builder
pub struct ComponentRegistration {
    info: ComponentInfo,
}

impl ComponentRegistration {
    pub fn new<T: Component>() -> Self {
        Self {
            info: ComponentInfo {
                type_id: TypeId::of::<T>(),
                type_name: T::type_name().to_string(),
                size: std::mem::size_of::<T>(),
                networked: false,
                migration_fn: None,
            },
        }
    }
    
    pub fn networked(mut self) -> Self {
        self.info.networked = true;
        self
    }
    
    pub fn migration<F>(mut self, f: F) -> Self 
    where
        F: Fn(&[u8]) -> LogicResult<Vec<u8>> + Send + Sync + 'static
    {
        self.info.migration_fn = Some(Arc::new(f));
        self
    }
    
    pub fn build(self) -> ComponentInfo {
        self.info
    }
}

/// Component registry for runtime type management
pub struct ComponentRegistry {
    components: Arc<RwLock<HashMap<TypeId, ComponentInfo>>>,
    by_name: Arc<RwLock<HashMap<String, TypeId>>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            by_name: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn register(&self, info: ComponentInfo) -> LogicResult<()> {
        let mut components = self.components.write();
        let mut by_name = self.by_name.write();
        
        components.insert(info.type_id, info.clone());
        by_name.insert(info.type_name.clone(), info.type_id);
        
        Ok(())
    }
    
    pub fn get(&self, type_id: TypeId) -> Option<ComponentInfo> {
        self.components.read().get(&type_id).cloned()
    }
    
    pub fn get_by_name(&self, name: &str) -> Option<ComponentInfo> {
        let by_name = self.by_name.read();
        if let Some(type_id) = by_name.get(name) {
            self.components.read().get(type_id).cloned()
        } else {
            None
        }
    }
    
    pub fn is_networked(&self, type_id: TypeId) -> bool {
        self.components
            .read()
            .get(&type_id)
            .map(|info| info.networked)
            .unwrap_or(false)
    }
    
    pub fn migrate(&self, type_id: TypeId, old_data: &[u8]) -> LogicResult<Vec<u8>> {
        let components = self.components.read();
        let info = components
            .get(&type_id)
            .ok_or_else(|| crate::error::LogicError::ComponentNotRegistered(format!("{:?}", type_id)))?;
        
        if let Some(migration_fn) = &info.migration_fn {
            migration_fn(old_data)
        } else {
            Ok(old_data.to_vec())
        }
    }
}

/// Dirty tracking for networked components
pub struct DirtyTracker {
    dirty_entities: Arc<RwLock<fnv::FnvHashSet<crate::entity::Entity>>>,
    dirty_components: Arc<RwLock<fnv::FnvHashMap<crate::entity::Entity, fnv::FnvHashSet<TypeId>>>>,
}

use fnv::{FnvHashMap, FnvHashSet};

impl DirtyTracker {
    pub fn new() -> Self {
        Self {
            dirty_entities: Arc::new(RwLock::new(FnvHashSet::default())),
            dirty_components: Arc::new(RwLock::new(FnvHashMap::default())),
        }
    }
    
    pub fn mark_dirty(&self, entity: crate::entity::Entity, component_type: TypeId) {
        self.dirty_entities.write().insert(entity);
        self.dirty_components
            .write()
            .entry(entity)
            .or_insert_with(FnvHashSet::default)
            .insert(component_type);
    }
    
    pub fn get_dirty_batch(&self, max_count: usize) -> Vec<(crate::entity::Entity, Vec<TypeId>)> {
        let mut dirty_entities = self.dirty_entities.write();
        let mut dirty_components = self.dirty_components.write();
        
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
    
    pub fn clear(&self) {
        self.dirty_entities.write().clear();
        self.dirty_components.write().clear();
    }
}