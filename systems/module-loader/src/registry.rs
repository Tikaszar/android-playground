//! Module registry for inter-module communication

use std::collections::HashMap;
use tokio::sync::RwLock;
use playground_api::Module;
use playground_core_types::{CoreResult, CoreError};

/// Registry for loaded modules
pub struct ModuleRegistry {
    /// Map of module name to module interface
    modules: RwLock<HashMap<String, ModuleHandle>>,
}

/// Handle to a module for registry
pub struct ModuleHandle {
    /// Module interface
    pub module: &'static Module,
    /// Module state pointer
    pub state: *mut u8,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            modules: RwLock::new(HashMap::new()),
        }
    }

    /// Register a module
    pub async fn register_module(
        &self,
        name: &str,
        module: &'static Module
    ) -> CoreResult<()> {
        let mut modules = self.modules.write().await;

        if modules.contains_key(name) {
            return Err(CoreError::ModuleLoadFailed(format!(
                "Module {} already registered",
                name
            )));
        }

        // Module state is managed by ModuleLoader, we just need the interface
        modules.insert(name.to_string(), ModuleHandle {
            module,
            state: std::ptr::null_mut(),
        });

        Ok(())
    }

    /// Unregister a module
    pub async fn unregister_module(&self, name: &str) -> CoreResult<()> {
        let mut modules = self.modules.write().await;
        modules.remove(name)
            .ok_or_else(|| CoreError::ModuleNotFound(name.to_string()))?;
        Ok(())
    }

    /// Call a method on a module
    pub async fn call(
        &self,
        module_name: &str,
        method: &str,
        args: &[u8]
    ) -> CoreResult<Vec<u8>> {
        let modules = self.modules.read().await;

        let handle = modules.get(module_name)
            .ok_or_else(|| CoreError::ModuleNotFound(module_name.to_string()))?;

        // Call the module method
        (handle.module.vtable.call)(handle.state, method, args)
            .map_err(|e| CoreError::Generic(format!(
                "Module call failed for {}::{}: {}",
                module_name,
                method,
                e
            )))
    }

    /// Get module capabilities
    pub async fn get_capabilities(&self, module_name: &str) -> CoreResult<Vec<String>> {
        let modules = self.modules.read().await;

        let handle = modules.get(module_name)
            .ok_or_else(|| CoreError::ModuleNotFound(module_name.to_string()))?;

        let caps = (handle.module.vtable.get_capabilities)();
        Ok(caps.iter().map(|s| s.to_string()).collect())
    }

    /// List all registered modules
    pub async fn list_modules(&self) -> Vec<String> {
        let modules = self.modules.read().await;
        modules.keys().cloned().collect()
    }

    /// Check if a module is registered
    pub async fn has_module(&self, name: &str) -> bool {
        let modules = self.modules.read().await;
        modules.contains_key(name)
    }
}

// Make registry thread-safe
unsafe impl Send for ModuleHandle {}
unsafe impl Sync for ModuleHandle {}