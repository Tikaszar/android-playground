use bytes::Bytes;
use serde::{Serialize, Deserialize};
use playground_core_types::{Shared, shared};
use crate::error::{LogicResult, LogicError};
use fnv::FnvHashMap;

/// Resource identifier using string instead of TypeId to avoid turbofish
pub type ResourceId = String;

/// Trait for types that can be used as resources
pub trait Resource: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// Get the unique resource ID for this type
    fn resource_id() -> ResourceId;
}

/// Concrete wrapper for resource data that avoids Box<dyn Any>
#[derive(Clone)]
pub struct ResourceData {
    data: Bytes,
    resource_id: ResourceId,
}

impl ResourceData {
    /// Create new ResourceData from a resource
    pub fn new<R: Resource>(resource: &R) -> LogicResult<Self> {
        let data = bincode::serialize(resource)
            .map_err(|e| LogicError::SerializationError(e.to_string()))?
            .into();
        
        Ok(Self {
            data,
            resource_id: R::resource_id(),
        })
    }
    
    /// Create ResourceData with explicit ID
    pub fn new_with_id(resource_id: ResourceId, data: Bytes) -> Self {
        Self {
            data,
            resource_id,
        }
    }
    
    /// Deserialize back to the original resource type
    pub fn deserialize<R: for<'de> Deserialize<'de>>(&self) -> LogicResult<R> {
        bincode::deserialize(&self.data)
            .map_err(|e| LogicError::SerializationError(e.to_string()))
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

/// Storage for global resources without Box<dyn Any>
pub struct ResourceStorage {
    resources: Shared<FnvHashMap<ResourceId, ResourceData>>,
}

impl ResourceStorage {
    /// Create new resource storage
    pub fn new() -> Self {
        Self {
            resources: shared(FnvHashMap::default()),
        }
    }
    
    /// Insert a resource with explicit ID
    pub async fn insert_with_id(&self, resource_id: ResourceId, data: Bytes) -> LogicResult<()> {
        let resource_data = ResourceData::new_with_id(resource_id.clone(), data);
        self.resources.write().await.insert(resource_id, resource_data);
        Ok(())
    }
    
    /// Insert a typed resource
    pub async fn insert<R: Resource>(&self, resource: R) -> LogicResult<()> {
        let resource_data = ResourceData::new(&resource)?;
        let resource_id = resource_data.resource_id().clone();
        
        self.resources.write().await.insert(resource_id, resource_data);
        Ok(())
    }
    
    /// Get a resource by ID and deserialize it
    pub async fn get<R: for<'de> Deserialize<'de>>(&self, resource_id: &ResourceId) -> Option<R> {
        let resources = self.resources.read().await;
        
        resources.get(resource_id)
            .and_then(|data| data.deserialize().ok())
    }
    
    /// Get a typed resource
    pub async fn get_typed<R: Resource>(&self) -> Option<R> {
        let resource_id = R::resource_id();
        self.get(&resource_id).await
    }
    
    /// Check if a resource exists by ID
    pub async fn contains(&self, resource_id: &ResourceId) -> bool {
        let resources = self.resources.read().await;
        resources.contains_key(resource_id)
    }
    
    /// Check if a typed resource exists
    pub async fn contains_typed<R: Resource>(&self) -> bool {
        let resource_id = R::resource_id();
        self.contains(&resource_id).await
    }
    
    /// Remove a resource by ID
    pub async fn remove<R: for<'de> Deserialize<'de>>(&self, resource_id: &ResourceId) -> Option<R> {
        let mut resources = self.resources.write().await;
        
        resources.remove(resource_id)
            .and_then(|data| data.deserialize().ok())
    }
    
    /// Remove a typed resource
    pub async fn remove_typed<R: Resource>(&self) -> Option<R> {
        let resource_id = R::resource_id();
        self.remove(&resource_id).await
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