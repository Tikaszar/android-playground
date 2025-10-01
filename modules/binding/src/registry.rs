//! Manages all View-ViewModel bindings

use crate::binding::Binding;
use crate::stats::BindingStats;
use playground_modules_types::{
    Handle, ModuleError, ModuleResult, Shared, ViewAPI, ViewModelFunction, ViewModelImpl,
};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Manages all View-ViewModel bindings
pub struct BindingRegistry {
    /// All active bindings indexed by View ID
    bindings: Shared<HashMap<String, Binding>>,

    /// View APIs waiting for ViewModel implementations
    pending_views: Shared<HashMap<String, ViewAPI>>,

    /// ViewModel implementations waiting for View APIs
    pending_viewmodels: Shared<HashMap<String, ViewModelImpl>>,
}

impl BindingRegistry {
    /// Create a new binding registry
    pub fn new() -> Self {
        Self {
            bindings: Handle::new(RwLock::new(HashMap::new())),
            pending_views: Handle::new(RwLock::new(HashMap::new())),
            pending_viewmodels: Handle::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a View API from a Core module
    pub async fn register_view(&self, view_id: String, api: ViewAPI) -> ModuleResult<()> {
        debug!("Registering View API: {}", view_id);

        // Check if there's a pending ViewModel for this View
        let mut pending_vms = self.pending_viewmodels.write().await;
        if let Some(viewmodel) = pending_vms.remove(&view_id) {
            // Bind immediately
            drop(pending_vms);
            self.create_binding(view_id.clone(), api, viewmodel).await?;
            info!("Immediately bound View {} to waiting ViewModel", view_id);
        } else {
            // Store as pending
            let mut pending = self.pending_views.write().await;
            pending.insert(view_id.clone(), api);
            debug!("View {} registered, waiting for ViewModel", view_id);
        }

        Ok(())
    }

    /// Register a ViewModel implementation from a System module
    pub async fn register_viewmodel(&self, viewmodel: ViewModelImpl) -> ModuleResult<()> {
        let view_id = viewmodel.view_id.to_string();
        debug!("Registering ViewModel for View: {}", view_id);

        // Check if there's a pending View for this ViewModel
        let mut pending_views = self.pending_views.write().await;
        if let Some(view_api) = pending_views.remove(&view_id) {
            // Bind immediately
            drop(pending_views);
            self.create_binding(view_id.clone(), view_api, viewmodel).await?;
            info!("Immediately bound ViewModel to waiting View {}", view_id);
        } else {
            // Store as pending
            let mut pending = self.pending_viewmodels.write().await;
            pending.insert(view_id.clone(), viewmodel);
            debug!("ViewModel for {} registered, waiting for View", view_id);
        }

        Ok(())
    }

    /// Create a binding between View and ViewModel
    async fn create_binding(
        &self,
        view_id: String,
        view_api: ViewAPI,
        viewmodel: ViewModelImpl,
    ) -> ModuleResult<()> {
        debug!("Creating binding for View: {}", view_id);

        // Validate that all View functions have ViewModel implementations
        let view_methods: Vec<&str> = view_api.functions.iter().map(|(name, _)| *name).collect();
        let vm_methods: Vec<&str> = viewmodel.functions.iter().map(|(name, _)| *name).collect();

        for method in &view_methods {
            if !vm_methods.contains(method) {
                return Err(ModuleError::BindingFailed(format!(
                    "ViewModel missing implementation for method: {}",
                    method
                )));
            }
        }

        // Create function map for direct calls
        let mut functions = HashMap::new();
        for (name, func) in viewmodel.functions {
            functions.insert(name.to_string(), *func);
        }

        // Store the binding
        let binding = Binding {
            view_id: view_id.clone(),
            functions,
        };

        let mut bindings = self.bindings.write().await;
        bindings.insert(view_id.clone(), binding);

        info!("Successfully bound View {} to ViewModel", view_id);
        Ok(())
    }

    /// Get a binding for direct function calls
    pub async fn get_binding(&self, view_id: &str) -> ModuleResult<HashMap<String, ViewModelFunction>> {
        let bindings = self.bindings.read().await;
        let binding = bindings
            .get(view_id)
            .ok_or_else(|| ModuleError::BindingFailed(format!("No binding for View: {}", view_id)))?;
        Ok(binding.functions.clone())
    }

    /// Call a function through binding (for testing)
    pub async fn call(
        &self,
        view_id: &str,
        method: &str,
        args: &[u8],
    ) -> ModuleResult<Vec<u8>> {
        let bindings = self.bindings.read().await;
        let binding = bindings
            .get(view_id)
            .ok_or_else(|| ModuleError::BindingFailed(format!("No binding for View: {}", view_id)))?;

        let function = binding
            .functions
            .get(method)
            .ok_or_else(|| ModuleError::BindingFailed(format!("Method not found: {}", method)))?;

        // Direct function call - no serialization overhead!
        let result = function(args).await?;
        Ok(result)
    }

    /// Unbind a View (for hot-reload)
    pub async fn unbind(&self, view_id: &str) -> ModuleResult<()> {
        let mut bindings = self.bindings.write().await;
        bindings.remove(view_id);
        debug!("Unbound View: {}", view_id);
        Ok(())
    }

    /// List all active bindings
    pub async fn list_bindings(&self) -> Vec<String> {
        let bindings = self.bindings.read().await;
        bindings.keys().cloned().collect()
    }

    /// Get binding statistics
    pub async fn get_stats(&self) -> BindingStats {
        let bindings = self.bindings.read().await;
        let pending_views = self.pending_views.read().await;
        let pending_viewmodels = self.pending_viewmodels.read().await;

        BindingStats {
            active_bindings: bindings.len(),
            pending_views: pending_views.len(),
            pending_viewmodels: pending_viewmodels.len(),
        }
    }
}

impl Default for BindingRegistry {
    fn default() -> Self {
        Self::new()
    }
}