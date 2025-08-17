use std::any::TypeId;
use std::sync::Arc;
use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use parking_lot::RwLock;
use crate::error::{EcsError, EcsResult};

pub type ComponentId = TypeId;

#[async_trait]
pub trait Component: Send + Sync + 'static {
    fn component_id() -> ComponentId where Self: Sized {
        TypeId::of::<Self>()
    }
    
    fn component_name() -> &'static str where Self: Sized {
        std::any::type_name::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes>;
    
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> where Self: Sized;
    
    fn size_hint(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

pub type ComponentBox = Box<dyn Component>;

#[derive(Clone)]
pub struct ComponentInfo {
    pub id: ComponentId,
    pub name: String,
    pub size_hint: usize,
    pub version: u32,
    pub networked: bool,
    pub migration_fn: Option<Arc<dyn Fn(&Bytes, u32) -> EcsResult<Bytes> + Send + Sync>>,
}

impl ComponentInfo {
    pub fn new<T: Component>() -> Self {
        Self {
            id: T::component_id(),
            name: T::component_name().to_string(),
            size_hint: std::mem::size_of::<T>(),
            version: 1,
            networked: false,
            migration_fn: None,
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
    
    pub fn with_migration<F>(mut self, f: F) -> Self 
    where
        F: Fn(&Bytes, u32) -> EcsResult<Bytes> + Send + Sync + 'static
    {
        self.migration_fn = Some(Arc::new(f));
        self
    }
}

pub struct ComponentRegistry {
    components: Arc<DashMap<ComponentId, ComponentInfo>>,
    name_to_id: Arc<DashMap<String, ComponentId>>,
    pool_size: Arc<RwLock<usize>>,
    pool_limit: usize,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self::with_pool_limit(1024 * 1024 * 100) // 100MB default
    }
    
    pub fn with_pool_limit(limit: usize) -> Self {
        Self {
            components: Arc::new(DashMap::new()),
            name_to_id: Arc::new(DashMap::new()),
            pool_size: Arc::new(RwLock::new(0)),
            pool_limit: limit,
        }
    }
    
    pub async fn register<T: Component>(&self) -> EcsResult<()> {
        self.register_with_info(ComponentInfo::new::<T>()).await
    }
    
    pub async fn register_with_info(&self, info: ComponentInfo) -> EcsResult<()> {
        let id = info.id;
        let name = info.name.clone();
        
        if self.components.contains_key(&id) {
            return Ok(());
        }
        
        self.components.insert(id, info.clone());
        self.name_to_id.insert(name, id);
        
        Ok(())
    }
    
    pub async fn get_info(&self, id: ComponentId) -> Option<ComponentInfo> {
        self.components.get(&id).map(|entry| entry.clone())
    }
    
    pub async fn get_info_by_name(&self, name: &str) -> Option<ComponentInfo> {
        self.name_to_id.get(name)
            .and_then(|id| self.components.get(&id.clone()))
            .map(|entry| entry.clone())
    }
    
    pub async fn allocate_pool_space(&self, size: usize) -> EcsResult<()> {
        let mut pool_size = self.pool_size.write();
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
        let mut pool_size = self.pool_size.write();
        *pool_size = pool_size.saturating_sub(size);
    }
    
    pub fn current_pool_usage(&self) -> usize {
        *self.pool_size.read()
    }
    
    pub fn pool_limit(&self) -> usize {
        self.pool_limit
    }
    
    pub fn pool_usage_percentage(&self) -> f32 {
        let usage = self.current_pool_usage() as f32;
        let limit = self.pool_limit as f32;
        (usage / limit) * 100.0
    }
    
    pub async fn is_networked(&self, id: ComponentId) -> bool {
        self.components
            .get(&id)
            .map(|info| info.networked)
            .unwrap_or(false)
    }
    
    pub async fn migrate_component(
        &self, 
        id: ComponentId, 
        data: &Bytes, 
        from_version: u32
    ) -> EcsResult<Bytes> {
        let info = self.get_info(id).await
            .ok_or_else(|| EcsError::ComponentNotRegistered(format!("{:?}", id)))?;
        
        if from_version == info.version {
            return Ok(data.clone());
        }
        
        if let Some(migration_fn) = &info.migration_fn {
            migration_fn(data, from_version)
        } else if cfg!(debug_assertions) {
            Ok(Bytes::new())
        } else {
            Err(EcsError::MigrationError(format!(
                "No migration from v{} to v{} for component {}",
                from_version, info.version, info.name
            )))
        }
    }
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
                return Err(EcsError::SerializationError("Invalid data".into()));
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