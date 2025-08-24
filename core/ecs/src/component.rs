use std::any::TypeId;
use std::collections::HashMap;
use bytes::Bytes;
use playground_core_types::{Shared, shared};
use crate::error::{EcsError, EcsResult};

pub type ComponentId = TypeId;

// Component is now a concrete base class that all components work through
pub struct Component {
    data: Bytes,
    component_id: ComponentId,
    component_name: String,
    size_hint: usize,
}

impl Component {
    pub fn new<T: ComponentData>(component: T) -> Self {
        let data = component.serialize();
        Self {
            data,
            component_id: T::component_id(),
            component_name: T::component_name().to_string(),
            size_hint: std::mem::size_of::<T>(),
        }
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
    
    pub fn deserialize<T: ComponentData>(&self) -> Result<T, EcsError> {
        T::deserialize(&self.data)
    }
    
    pub fn size_hint(&self) -> usize {
        self.size_hint
    }
}

// Trait for actual component data types
pub trait ComponentData: Send + Sync + 'static {
    fn component_id() -> ComponentId where Self: Sized {
        TypeId::of::<Self>()
    }
    
    fn component_name() -> &'static str where Self: Sized {
        std::any::type_name::<Self>()
    }
    
    fn serialize(&self) -> Bytes;
    
    fn deserialize(bytes: &Bytes) -> Result<Self, EcsError> where Self: Sized;
}

pub type ComponentBox = Box<Component>;

#[derive(Clone)]
pub struct ComponentInfo {
    pub id: ComponentId,
    pub name: String,
    pub size_hint: usize,
    pub version: u32,
    pub networked: bool,
}

impl ComponentInfo {
    pub fn new<T: ComponentData>() -> Self {
        Self {
            id: T::component_id(),
            name: T::component_name().to_string(),
            size_hint: std::mem::size_of::<T>(),
            version: 1,
            networked: false,
        }
    }
    
    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }
    
    pub fn networked(mut self) -> Self {
        self.networked = true;
        self
    }
}

pub struct ComponentRegistry {
    components: Shared<HashMap<ComponentId, ComponentInfo>>,
    name_to_id: Shared<HashMap<String, ComponentId>>,
    pool_size: Shared<usize>,
    pool_limit: usize,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self::with_pool_limit(1024 * 1024 * 100) // 100MB default
    }
    
    pub fn with_pool_limit(limit: usize) -> Self {
        Self {
            components: shared(HashMap::new()),
            name_to_id: shared(HashMap::new()),
            pool_size: shared(0),
            pool_limit: limit,
        }
    }
    
    pub async fn register<T: ComponentData>(&self) -> EcsResult<()> {
        self.register_with_info(ComponentInfo::new::<T>()).await
    }
    
    pub async fn register_with_info(&self, info: ComponentInfo) -> EcsResult<()> {
        let id = info.id;
        let name = info.name.clone();
        
        if self.components.read().await.contains_key(&id) {
            return Ok(());
        }
        
        self.components.write().await.insert(id, info.clone());
        self.name_to_id.write().await.insert(name, id);
        
        Ok(())
    }
    
    pub async fn get_info(&self, id: ComponentId) -> Option<ComponentInfo> {
        self.components.read().await.get(&id).cloned()
    }
    
    pub async fn get_info_by_name(&self, name: &str) -> Option<ComponentInfo> {
        let name_to_id = self.name_to_id.read().await;
        if let Some(id) = name_to_id.get(name) {
            self.components.read().await.get(id).cloned()
        } else {
            None
        }
    }
    
    pub async fn allocate_pool_space(&self, size: usize) -> EcsResult<()> {
        let mut pool_size = self.pool_size.write().await;
        let new_size = *pool_size + size;
        
        if new_size > self.pool_limit {
            return Err(EcsError::MemoryLimitExceeded {
                current: new_size,
                limit: self.pool_limit,
            });
        }
        
        *pool_size = new_size;
        Ok(())
    }
    
    pub async fn free_pool_space(&self, size: usize) {
        let mut pool_size = self.pool_size.write().await;
        *pool_size = pool_size.saturating_sub(size);
    }
    
    pub async fn current_pool_usage(&self) -> usize {
        *self.pool_size.read().await
    }
    
    pub fn pool_limit(&self) -> usize {
        self.pool_limit
    }
    
    pub async fn pool_usage_percentage(&self) -> f32 {
        let usage = self.current_pool_usage().await as f32;
        let limit = self.pool_limit as f32;
        (usage / limit) * 100.0
    }
    
    pub async fn is_networked(&self, id: ComponentId) -> bool {
        self.components
            .read()
            .await
            .get(&id)
            .map(|info| info.networked)
            .unwrap_or(false)
    }
    
    // Migration removed - not needed for NO dyn architecture
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestComponent {
        value: i32,
    }
    
    #[async_trait]
    impl Component for TestComponent {
        async fn serialize(&self) -> EcsResult<Bytes> {
            let mut buf = BytesMut::new();
            buf.extend_from_slice(&self.value.to_le_bytes());
            Ok(buf.freeze())
        }
        
        async fn deserialize(bytes: &Bytes) -> EcsResult<Self> {
            if bytes.len() < 4 {
                return Err(EcsError::SerializationFailed("Invalid data".into()));
            }
            let value = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            Ok(Self { value })
        }
    }
    
    #[tokio::test]
    async fn test_component_registration() {
        let registry = ComponentRegistry::new();
        registry.register::<TestComponent>().await.unwrap();
        
        let info = registry.get_info(TestComponent::component_id()).await;
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, std::any::type_name::<TestComponent>());
    }
    
    #[tokio::test]
    async fn test_pool_allocation() {
        let registry = ComponentRegistry::with_pool_limit(1000);
        
        assert!(registry.allocate_pool_space(500).await.is_ok());
        assert_eq!(registry.current_pool_usage(), 500);
        
        assert!(registry.allocate_pool_space(600).await.is_err());
        
        registry.free_pool_space(200).await;
        assert_eq!(registry.current_pool_usage(), 300);
    }
}