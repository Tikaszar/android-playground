use bytes::Bytes;
use std::any::TypeId;
use crate::error::LogicResult;
use crate::world::World;
use crate::system::{System, SystemInfo};
use async_trait::async_trait;

/// Internal trait for system execution without exposing dyn
#[async_trait]
trait SystemExecutor: Send + Sync {
    fn name(&self) -> &'static str;
    fn type_id(&self) -> TypeId;
    fn dependencies(&self) -> Vec<TypeId>;
    fn parallel(&self) -> bool;
    async fn initialize(&mut self, world: &World) -> LogicResult<()>;
    async fn run(&mut self, world: &World, delta_time: f32) -> LogicResult<()>;
    async fn cleanup(&mut self, world: &World) -> LogicResult<()>;
    fn as_bytes(&self) -> LogicResult<Bytes>;
}

/// Concrete wrapper for each system type
struct TypedSystem<S: System> {
    system: S,
}

#[async_trait]
impl<S: System> SystemExecutor for TypedSystem<S> {
    fn name(&self) -> &'static str {
        self.system.name()
    }
    
    fn type_id(&self) -> TypeId {
        TypeId::of::<S>()
    }
    
    fn dependencies(&self) -> Vec<TypeId> {
        self.system.dependencies()
    }
    
    fn parallel(&self) -> bool {
        self.system.parallel()
    }
    
    async fn initialize(&mut self, world: &World) -> LogicResult<()> {
        self.system.initialize(world).await
    }
    
    async fn run(&mut self, world: &World, delta_time: f32) -> LogicResult<()> {
        self.system.run(world, delta_time).await
    }
    
    async fn cleanup(&mut self, world: &World) -> LogicResult<()> {
        self.system.cleanup(world).await
    }
    
    fn as_bytes(&self) -> LogicResult<Bytes> {
        // Systems are not serializable in general, return empty
        Ok(Bytes::new())
    }
}

/// Concrete wrapper for systems that avoids Box<dyn System>
pub struct SystemData {
    inner: Box<dyn SystemExecutor>,
    info: SystemInfo,
}

impl SystemData {
    /// Create new SystemData from a system
    pub fn new<S: System>(system: S) -> Self {
        let name = system.name().to_string();
        let type_id = TypeId::of::<S>();
        let dependencies = system.dependencies();
        let parallel = system.parallel();
        
        let info = SystemInfo {
            type_id,
            name,
            dependencies,
            parallel,
            retry_count: 0,
            max_retries: 3,
            enabled: true,
            safe_mode: false,
        };
        
        Self {
            inner: Box::new(TypedSystem { system }),
            info,
        }
    }
    
    /// Get system name
    pub fn name(&self) -> &str {
        &self.info.name
    }
    
    /// Get system type ID
    pub fn type_id(&self) -> TypeId {
        self.info.type_id
    }
    
    /// Get system dependencies
    pub fn dependencies(&self) -> &[TypeId] {
        &self.info.dependencies
    }
    
    /// Check if system can run in parallel
    pub fn parallel(&self) -> bool {
        self.info.parallel
    }
    
    /// Get mutable reference to system info
    pub fn info_mut(&mut self) -> &mut SystemInfo {
        &mut self.info
    }
    
    /// Get reference to system info
    pub fn info(&self) -> &SystemInfo {
        &self.info
    }
    
    /// Initialize the system
    pub async fn initialize(&mut self, world: &World) -> LogicResult<()> {
        self.inner.initialize(world).await
    }
    
    /// Run the system
    pub async fn run(&mut self, world: &World, delta_time: f32) -> LogicResult<()> {
        if !self.info.enabled {
            return Ok(());
        }
        self.inner.run(world, delta_time).await
    }
    
    /// Cleanup the system
    pub async fn cleanup(&mut self, world: &World) -> LogicResult<()> {
        self.inner.cleanup(world).await
    }
    
    /// Set last error
    pub fn set_last_error(&mut self, error: Option<String>) {
        // Store in info if we add that field
    }
    
    /// Check if system is enabled
    pub fn is_enabled(&self) -> bool {
        self.info.enabled
    }
    
    /// Enable or disable the system
    pub fn set_enabled(&mut self, enabled: bool) {
        self.info.enabled = enabled;
    }
}