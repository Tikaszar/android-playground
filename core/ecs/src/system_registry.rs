use playground_core_types::{Handle, handle, Shared, shared};
use std::collections::HashMap;
use crate::error::{EcsError, EcsResult};

/// Registry for managing system instances
/// This maintains the NO dyn rule by storing concrete types
pub struct SystemRegistry {
    // Store systems by type name
    systems: Shared<HashMap<String, SystemHandle>>,
}

/// Handle to a registered system
/// This wraps the system to avoid dyn while allowing storage
pub struct SystemHandle {
    pub name: String,
    pub system_type: String,
    // The actual system instance is managed by the specific system type
    // We just track that it exists
    pub initialized: bool,
}

impl SystemRegistry {
    /// Create a new system registry
    pub fn new() -> Self {
        Self {
            systems: shared(HashMap::new()),
        }
    }
    
    /// Register a network system
    pub async fn register_network_system(&self, name: String) -> EcsResult<()> {
        let mut systems = self.systems.write().await;
        
        let handle = SystemHandle {
            name: name.clone(),
            system_type: "network".to_string(),
            initialized: false,
        };
        
        systems.insert(name, handle);
        Ok(())
    }
    
    /// Register a UI system
    pub async fn register_ui_system(&self, name: String) -> EcsResult<()> {
        let mut systems = self.systems.write().await;
        
        let handle = SystemHandle {
            name: name.clone(),
            system_type: "ui".to_string(),
            initialized: false,
        };
        
        systems.insert(name, handle);
        Ok(())
    }
    
    /// Register a render system
    pub async fn register_render_system(&self, name: String) -> EcsResult<()> {
        let mut systems = self.systems.write().await;
        
        let handle = SystemHandle {
            name: name.clone(),
            system_type: "render".to_string(),
            initialized: false,
        };
        
        systems.insert(name, handle);
        Ok(())
    }
    
    /// Register a physics system
    pub async fn register_physics_system(&self, name: String) -> EcsResult<()> {
        let mut systems = self.systems.write().await;
        
        let handle = SystemHandle {
            name: name.clone(),
            system_type: "physics".to_string(),
            initialized: false,
        };
        
        systems.insert(name, handle);
        Ok(())
    }
    
    /// Mark all registered systems as initialized
    pub async fn initialize_all_registered(&self) -> EcsResult<()> {
        let mut systems = self.systems.write().await;
        
        for handle in systems.values_mut() {
            handle.initialized = true;
        }
        
        Ok(())
    }
    
    /// Check if a system is registered
    pub async fn is_registered(&self, name: &str) -> bool {
        let systems = self.systems.read().await;
        systems.contains_key(name)
    }
    
    /// Get list of all registered systems
    pub async fn get_registered_systems(&self) -> Vec<String> {
        let systems = self.systems.read().await;
        systems.keys().cloned().collect()
    }
}

use once_cell::sync::Lazy;

// Global registry instance using Lazy for thread-safe initialization
static GLOBAL_REGISTRY: Lazy<SystemRegistry> = Lazy::new(|| {
    SystemRegistry::new()
});

/// Register a network system with the global registry
pub async fn register_network_system(name: String) -> EcsResult<()> {
    GLOBAL_REGISTRY.register_network_system(name).await
}

/// Register a UI system with the global registry
pub async fn register_ui_system(name: String) -> EcsResult<()> {
    GLOBAL_REGISTRY.register_ui_system(name).await
}

/// Register a render system with the global registry
pub async fn register_render_system(name: String) -> EcsResult<()> {
    GLOBAL_REGISTRY.register_render_system(name).await
}

/// Register a physics system with the global registry
pub async fn register_physics_system(name: String) -> EcsResult<()> {
    GLOBAL_REGISTRY.register_physics_system(name).await
}

/// Initialize all registered systems
pub async fn initialize_all_registered() -> EcsResult<()> {
    GLOBAL_REGISTRY.initialize_all_registered().await
}

/// Check if a system is registered
pub async fn is_system_registered(name: &str) -> bool {
    GLOBAL_REGISTRY.is_registered(name).await
}

/// Get list of all registered systems
pub async fn get_registered_systems() -> Vec<String> {
    GLOBAL_REGISTRY.get_registered_systems().await
}