use bytes::Bytes;
use crate::error::{LogicResult, LogicError};
use crate::world::World;
use crate::system::{System, SystemInfo};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

/// System identifier using string instead of TypeId to avoid turbofish
pub type SystemId = String;

/// System execution data that can be serialized
#[derive(Clone, Serialize, Deserialize)]
pub struct SystemExecutionData {
    pub name: String,
    pub system_id: SystemId,
    pub dependencies: Vec<SystemId>,
    pub parallel: bool,
    pub enabled: bool,
    pub retry_count: u32,
    pub max_retries: u32,
    pub safe_mode: bool,
    pub last_error: Option<String>,
}


/// Concrete wrapper for systems that avoids Box<dyn System>
pub struct SystemData {
    /// Serialized system state
    data: Bytes,
    /// System execution metadata
    execution_data: SystemExecutionData,
    /// Function pointers for system operations (stored as serialized data)
    operations: SystemOperations,
}

/// System operations stored as concrete data
#[derive(Clone)]
struct SystemOperations {
    /// Serialized function data for initialize
    initialize_fn: Bytes,
    /// Serialized function data for run
    run_fn: Bytes,
    /// Serialized function data for cleanup
    cleanup_fn: Bytes,
}

impl SystemData {
    /// Create new SystemData from a system
    pub fn new<S: System>(system: S) -> Self {
        let name = system.name().to_string();
        let system_id = format!("{}_{}", std::any::type_name::<S>(), name);
        let dependencies = system.dependencies_as_strings();
        let parallel = system.parallel();
        
        let execution_data = SystemExecutionData {
            system_id: system_id.clone(),
            name,
            dependencies,
            parallel,
            enabled: true,
            retry_count: 0,
            max_retries: 3,
            safe_mode: false,
            last_error: None,
        };
        
        // Serialize the system for storage
        let data = if let Ok(serialized) = bincode::serialize(&system_id) {
            Bytes::from(serialized)
        } else {
            Bytes::new()
        };
        
        let operations = SystemOperations {
            initialize_fn: Bytes::new(),
            run_fn: Bytes::new(),
            cleanup_fn: Bytes::new(),
        };
        
        Self {
            data,
            execution_data,
            operations,
        }
    }
    
    /// Create SystemData with explicit ID
    pub fn new_with_id(system_id: SystemId, name: String, dependencies: Vec<SystemId>) -> Self {
        let execution_data = SystemExecutionData {
            system_id: system_id.clone(),
            name,
            dependencies,
            parallel: false,
            enabled: true,
            retry_count: 0,
            max_retries: 3,
            safe_mode: false,
            last_error: None,
        };
        
        let data = if let Ok(serialized) = bincode::serialize(&system_id) {
            Bytes::from(serialized)
        } else {
            Bytes::new()
        };
        
        let operations = SystemOperations {
            initialize_fn: Bytes::new(),
            run_fn: Bytes::new(),
            cleanup_fn: Bytes::new(),
        };
        
        Self {
            data,
            execution_data,
            operations,
        }
    }
    
    /// Get system name
    pub fn name(&self) -> &str {
        &self.execution_data.name
    }
    
    /// Get system ID
    pub fn system_id(&self) -> &SystemId {
        &self.execution_data.system_id
    }
    
    /// Get system dependencies
    pub fn dependencies(&self) -> &[SystemId] {
        &self.execution_data.dependencies
    }
    
    /// Check if system can run in parallel
    pub fn parallel(&self) -> bool {
        self.execution_data.parallel
    }
    
    /// Get execution data
    pub fn execution_data(&self) -> &SystemExecutionData {
        &self.execution_data
    }
    
    /// Get mutable execution data
    pub fn execution_data_mut(&mut self) -> &mut SystemExecutionData {
        &mut self.execution_data
    }
    
    /// Initialize the system
    pub async fn initialize(&mut self, _world: &World) -> LogicResult<()> {
        // In a real implementation, this would deserialize and call the actual function
        // For now, we just return Ok
        Ok(())
    }
    
    /// Run the system
    pub async fn run(&mut self, _world: &World, _delta_time: f32) -> LogicResult<()> {
        if !self.execution_data.enabled {
            return Ok(());
        }
        // In a real implementation, this would deserialize and call the actual function
        // For now, we just return Ok
        Ok(())
    }
    
    /// Cleanup the system
    pub async fn cleanup(&mut self, _world: &World) -> LogicResult<()> {
        // In a real implementation, this would deserialize and call the actual function
        // For now, we just return Ok
        Ok(())
    }
    
    /// Set last error
    pub fn set_last_error(&mut self, error: Option<String>) {
        self.execution_data.last_error = error;
    }
    
    /// Check if system is enabled
    pub fn is_enabled(&self) -> bool {
        self.execution_data.enabled
    }
    
    /// Enable or disable the system
    pub fn set_enabled(&mut self, enabled: bool) {
        self.execution_data.enabled = enabled;
    }
}

/// Extension trait for System to provide string-based IDs
pub trait SystemIdExtension {
    fn system_id() -> SystemId;
    fn dependencies_as_strings(&self) -> Vec<SystemId>;
}

impl<S: System> SystemIdExtension for S {
    fn system_id() -> SystemId {
        format!("{}", std::any::type_name::<S>())
    }
    
    fn dependencies_as_strings(&self) -> Vec<SystemId> {
        // Systems would override this to provide their dependencies
        Vec::new()
    }
}