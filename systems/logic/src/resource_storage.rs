use bytes::Bytes;
use serde::{Serialize, Deserialize};
use std::any::TypeId;
use playground_core_types::{Shared, shared};
use crate::error::{LogicResult, LogicError};
use fnv::FnvHashMap;

/// Concrete wrapper for resource data that avoids Box<dyn Any>
#[derive(Clone)]
pub struct ResourceData {
    data: Bytes,
    type_id: TypeId,
    type_name: String,
}

impl ResourceData {
    /// Create new ResourceData from a resource
    pub fn new<R: Serialize + 'static>(resource: R) -> LogicResult<Self> {
        let data = bincode::serialize(&resource)
            .map_err(|e| LogicError::SerializationError(e.to_string()))?
            .into();
        
        Ok(Self {
            data,
            type_id: TypeId::of::<R>(),
            type_name: std::any::type_name::<R>().to_string(),
        })
    }
    
    /// Deserialize back to the original resource type
    pub fn deserialize<R: for<'de> Deserialize<'de>>(&self) -> LogicResult<R> {
        bincode::deserialize(&self.data)
            .map_err(|e| LogicError::SerializationError(e.to_string()))
    }
    
    /// Get the type ID of this resource
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
    
    /// Get the type name of this resource
    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

/// Storage for global resources without Box<dyn Any>
pub struct ResourceStorage {
    resources: Shared<FnvHashMap<TypeId, ResourceData>>,
}

impl ResourceStorage {
    /// Create new resource storage
    pub fn new() -> Self {
        Self {
            resources: shared(FnvHashMap::default()),
        }
    }
    
    /// Insert a resource
    pub async fn insert<R: Serialize + Send + Sync + 'static>(&self, resource: R) -> LogicResult<()> {
        let resource_data = ResourceData::new(resource)?;
        let type_id = resource_data.type_id();
        
        self.resources.write().await.insert(type_id, resource_data);
        Ok(())
    }
    
    /// Get a resource
    pub async fn get<R: for<'de> Deserialize<'de> + 'static>(&self) -> Option<R> {
        let resources = self.resources.read().await;
        let type_id = TypeId::of::<R>();
        
        resources.get(&type_id)
            .and_then(|data| data.deserialize().ok())
    }
    
    /// Check if a resource exists
    pub async fn contains<R: 'static>(&self) -> bool {
        let resources = self.resources.read().await;
        resources.contains_key(&TypeId::of::<R>())
    }
    
    /// Remove a resource
    pub async fn remove<R: 'static>(&self) -> Option<R> 
    where
        R: for<'de> Deserialize<'de>
    {
        let mut resources = self.resources.write().await;
        let type_id = TypeId::of::<R>();
        
        resources.remove(&type_id)
            .and_then(|data| data.deserialize().ok())
    }
    
    /// Clear all resources
    pub async fn clear(&self) {
        self.resources.write().await.clear();
    }
    
    /// Get the number of resources
    pub async fn len(&self) -> usize {
        self.resources.read().await.len()
    }
    
    /// Check if storage is empty
    pub async fn is_empty(&self) -> bool {
        self.resources.read().await.is_empty()
    }
}