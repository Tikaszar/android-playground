//! Component management for the unified ECS

use std::collections::HashMap;
use bytes::Bytes;
use playground_core_types::{Shared, shared};
use playground_core_ecs::{ComponentData, ComponentId, EcsError, EcsResult};

/// Component wrapper that provides type erasure without dyn
/// This is the concrete base class that all components work through
#[derive(Clone)]
pub struct Component {
    data: Bytes,
    component_id: ComponentId,
    component_name: String,
    size_hint: usize,
}

impl Component {
    /// Create a new component from typed data
    pub async fn new<T: ComponentData>(component: T) -> EcsResult<Self> {
        let data = component.serialize().await?;
        Ok(Self {
            data,
            component_id: T::component_id(),
            component_name: T::component_name().to_string(),
            size_hint: 0, // Size hint removed to avoid turbofish
        })
    }
    
    /// Create a component from raw bytes
    pub fn from_bytes(data: Bytes, component_id: ComponentId, component_name: String, size_hint: usize) -> Self {
        Self {
            data,
            component_id,
            component_name,
            size_hint,
        }
    }
    
    /// Get the component ID
    pub fn component_id(&self) -> ComponentId {
        self.component_id.clone()
    }
    
    /// Get the component name
    pub fn component_name(&self) -> &str {
        &self.component_name
    }
    
    /// Serialize the component to bytes
    pub fn serialize(&self) -> Bytes {
        self.data.clone()
    }
    
    /// Deserialize the component to a typed value
    pub async fn deserialize<T: ComponentData>(&self) -> EcsResult<T> {
        T::deserialize(&self.data).await
    }
}

/// Type alias for boxed components (NO dyn pattern)
pub type ComponentBox = Box<Component>;

/// Helper functions for ComponentBox
pub mod component_box {
    use super::*;
    
    /// Create a ComponentBox from raw bytes
    pub fn from_bytes(component_id: ComponentId, data: Bytes) -> ComponentBox {
        Box::new(Component::from_bytes(
            data,
            component_id.clone(),
            format!("{:?}", component_id), // Use Debug format as name
            0, // Size hint not needed
        ))
    }
}

/// Component information metadata
#[derive(Clone)]
pub struct ComponentInfo {
    pub id: ComponentId,
    pub name: String,
    pub size_hint: usize,
    pub version: u32,
    pub networked: bool,
}

impl ComponentInfo {
    /// Create component info from a component type
    pub fn new<T: ComponentData>() -> Self {
        Self {
            id: T::component_id(),
            name: T::component_name().to_string(),
            size_hint: 0, // Size hint removed to avoid turbofish
            version: 1,
            networked: false,
        }
    }
    
    /// Set the version
    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }
    
    /// Mark as networked
    pub fn networked(mut self) -> Self {
        self.networked = true;
        self
    }
}

/// Component registry manages component types and metadata
pub struct ComponentRegistry {
    components: Shared<HashMap<ComponentId, ComponentInfo>>,
    name_to_id: Shared<HashMap<String, ComponentId>>,
    pool_size: Shared<usize>,
    pool_limit: usize,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self::with_pool_limit(1024 * 1024 * 100) // 100MB default
    }
    
    /// Create a registry with a custom pool limit
    pub fn with_pool_limit(limit: usize) -> Self {
        Self {
            components: shared(HashMap::new()),
            name_to_id: shared(HashMap::new()),
            pool_size: shared(0),
            pool_limit: limit,
        }
    }
    
    /// Register a component type
    pub async fn register<T: ComponentData>(&self) -> EcsResult<()> {
        self.register_with_info(ComponentInfo::new::<T>()).await
    }
    
    /// Register a component with custom info
    pub async fn register_with_info(&self, info: ComponentInfo) -> EcsResult<()> {
        let id = info.id.clone();
        let name = info.name.clone();
        
        if self.components.read().await.contains_key(&id) {
            return Ok(()); // Already registered
        }
        
        // Check pool limit
        let new_size = *self.pool_size.read().await + info.size_hint;
        if new_size > self.pool_limit {
            return Err(EcsError::PoolExhausted(format!(
                "Pool exhausted: requested {}, available {}",
                info.size_hint,
                self.pool_limit - *self.pool_size.read().await
            )));
        }
        
        self.components.write().await.insert(id.clone(), info);
        self.name_to_id.write().await.insert(name, id);
        *self.pool_size.write().await = new_size;
        
        Ok(())
    }
    
    /// Lookup component info by ID
    pub async fn get_info(&self, id: &ComponentId) -> Option<ComponentInfo> {
        self.components.read().await.get(id).cloned()
    }
    
    /// Lookup component ID by name
    pub async fn get_id(&self, name: &str) -> Option<ComponentId> {
        self.name_to_id.read().await.get(name).cloned()
    }
    
    /// Check if a component is registered
    pub async fn is_registered(&self, id: &ComponentId) -> bool {
        self.components.read().await.contains_key(id)
    }
    
    /// Get all registered component IDs
    pub async fn all_ids(&self) -> Vec<ComponentId> {
        self.components.read().await.keys().cloned().collect()
    }
    
    /// Get current pool usage
    pub async fn current_pool_usage(&self) -> usize {
        *self.pool_size.read().await
    }
    
    /// Get pool limit
    pub fn pool_limit(&self) -> usize {
        self.pool_limit
    }
    
    /// Clear all registrations
    pub async fn clear(&self) {
        self.components.write().await.clear();
        self.name_to_id.write().await.clear();
        *self.pool_size.write().await = 0;
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}