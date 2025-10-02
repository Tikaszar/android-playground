//! Manages all Model/View/ViewModel bindings with sharded storage

use crate::pool::ModelPool;
use crate::stats::BindingStats;
use playground_modules_types::{
    Handle, ModelId, ModelTrait, ModelType, ModuleError, ModuleResult, ViewId, ViewModelTrait,
    ViewTrait,
};
use std::collections::HashMap;
use tracing::{debug, info};

/// Manages all MVVM bindings with triple-nested sharding
///
/// Storage architecture:
/// - Views: Immutable HashMap (lock-free reads)
/// - ViewModels: Immutable HashMap (lock-free reads)
/// - Models: ViewId -> ModelType -> Pool (fine-grained locking)
pub struct BindingRegistry {
    /// Views indexed by ViewId (lock-free singleton access)
    views: Handle<HashMap<ViewId, Handle<dyn ViewTrait>>>,

    /// ViewModels indexed by ViewId (lock-free singleton access)
    viewmodels: Handle<HashMap<ViewId, Handle<dyn ViewModelTrait>>>,

    /// Models with triple-nested sharding:
    /// ViewId -> ModelType -> ModelPool (RwLock at pool level only)
    models: Handle<HashMap<ViewId, Handle<HashMap<ModelType, ModelPool>>>>,
}

impl BindingRegistry {
    /// Create a new binding registry
    pub fn new() -> Self {
        Self {
            views: Handle::new(HashMap::new()),
            viewmodels: Handle::new(HashMap::new()),
            models: Handle::new(HashMap::new()),
        }
    }

    // ========================================================================
    // View Operations (Singleton, Lock-Free Reads)
    // ========================================================================

    /// Register a View (replaces if exists)
    pub fn register_view(&mut self, view: Handle<dyn ViewTrait>) {
        let view_id = view.view_id();
        debug!("Registering View: {:#018x}", view_id);

        // Clone HashMap, insert, swap Handle (immutable update)
        let mut new_views = (*self.views).clone();
        new_views.insert(view_id, view);
        self.views = Handle::new(new_views);

        info!("Registered View: {:#018x}", view_id);
    }

    /// Get a View by ID (lock-free)
    pub fn get_view(&self, view_id: ViewId) -> Option<Handle<dyn ViewTrait>> {
        self.views.get(&view_id).cloned()
    }

    // ========================================================================
    // ViewModel Operations (Singleton, Lock-Free Reads)
    // ========================================================================

    /// Bind a ViewModel to its View (replaces if exists)
    pub fn bind_viewmodel(&mut self, viewmodel: Handle<dyn ViewModelTrait>) -> ModuleResult<()> {
        let view_id = viewmodel.view_id();
        debug!("Binding ViewModel for View: {:#018x}", view_id);

        // Verify View exists
        if !self.views.contains_key(&view_id) {
            return Err(ModuleError::BindingFailed(format!(
                "View not found: {:#018x}",
                view_id
            )));
        }

        // Clone HashMap, insert, swap Handle (immutable update)
        let mut new_viewmodels = (*self.viewmodels).clone();
        new_viewmodels.insert(view_id, viewmodel);
        self.viewmodels = Handle::new(new_viewmodels);

        info!("Bound ViewModel for View: {:#018x}", view_id);
        Ok(())
    }

    /// Get a ViewModel by View ID (lock-free)
    pub fn get_viewmodel(&self, view_id: ViewId) -> Option<Handle<dyn ViewModelTrait>> {
        self.viewmodels.get(&view_id).cloned()
    }

    /// Unbind a ViewModel (for hot-reload)
    pub fn unbind_viewmodel(&mut self, view_id: ViewId) -> ModuleResult<()> {
        debug!("Unbinding ViewModel for View: {:#018x}", view_id);

        // Clone HashMap, remove, swap Handle
        let mut new_viewmodels = (*self.viewmodels).clone();
        if new_viewmodels.remove(&view_id).is_some() {
            self.viewmodels = Handle::new(new_viewmodels);
            info!("Unbound ViewModel for View: {:#018x}", view_id);
            Ok(())
        } else {
            Err(ModuleError::NotFound(format!(
                "ViewModel not found: {:#018x}",
                view_id
            )))
        }
    }

    // ========================================================================
    // Model Pool Operations (Triple-Nested Sharding)
    // ========================================================================

    /// Register a model pool for a specific View and ModelType
    pub fn register_pool(&mut self, view_id: ViewId, model_type: ModelType, pool: ModelPool) {
        debug!(
            "Registering pool for View {:#018x}, ModelType {:#018x}",
            view_id, model_type
        );

        // Get or create View's model map
        let view_models = self
            .models
            .get(&view_id)
            .cloned()
            .unwrap_or_else(|| Handle::new(HashMap::new()));

        // Clone map, insert pool, update
        let mut new_type_map = (*view_models).clone();
        new_type_map.insert(model_type, pool);

        // Clone outer map, insert view's map, update
        let mut new_models = (*self.models).clone();
        new_models.insert(view_id, Handle::new(new_type_map));
        self.models = Handle::new(new_models);

        info!(
            "Registered pool for View {:#018x}, ModelType {:#018x}",
            view_id, model_type
        );
    }

    /// Get a model pool (lock-free until pool access)
    pub fn get_pool(&self, view_id: ViewId, model_type: ModelType) -> Option<&ModelPool> {
        self.models
            .get(&view_id)?
            .get(&model_type)
    }

    // ========================================================================
    // Model Operations (Delegates to Pool)
    // ========================================================================

    /// Create a model in a pool
    pub async fn create_model(
        &self,
        view_id: ViewId,
        model_type: ModelType,
        model_id: ModelId,
        model: Handle<dyn ModelTrait>,
    ) -> ModuleResult<()> {
        let pool = self.get_pool(view_id, model_type).ok_or_else(|| {
            ModuleError::NotFound(format!(
                "Pool not found for View {:#018x}, ModelType {:#018x}",
                view_id, model_type
            ))
        })?;

        pool.insert(model_id, model).await;
        Ok(())
    }

    /// Get a model from a pool
    pub async fn get_model(
        &self,
        view_id: ViewId,
        model_type: ModelType,
        model_id: ModelId,
    ) -> ModuleResult<Handle<dyn ModelTrait>> {
        let pool = self.get_pool(view_id, model_type).ok_or_else(|| {
            ModuleError::NotFound(format!(
                "Pool not found for View {:#018x}, ModelType {:#018x}",
                view_id, model_type
            ))
        })?;

        pool.get(model_id)
            .await
            .ok_or_else(|| ModuleError::NotFound(format!("Model not found: {:#018x}", model_id)))
    }

    /// Delete a model (moves to recycle pool)
    pub async fn delete_model(
        &self,
        view_id: ViewId,
        model_type: ModelType,
        model_id: ModelId,
    ) -> ModuleResult<()> {
        let pool = self.get_pool(view_id, model_type).ok_or_else(|| {
            ModuleError::NotFound(format!(
                "Pool not found for View {:#018x}, ModelType {:#018x}",
                view_id, model_type
            ))
        })?;

        pool.remove(model_id).await;
        Ok(())
    }

    /// Get or recycle a model (prefer reused objects)
    pub async fn get_or_recycle_model<F>(
        &self,
        view_id: ViewId,
        model_type: ModelType,
        model_id: ModelId,
        factory: F,
    ) -> ModuleResult<Handle<dyn ModelTrait>>
    where
        F: FnOnce() -> Handle<dyn ModelTrait>,
    {
        let pool = self.get_pool(view_id, model_type).ok_or_else(|| {
            ModuleError::NotFound(format!(
                "Pool not found for View {:#018x}, ModelType {:#018x}",
                view_id, model_type
            ))
        })?;

        Ok(pool.get_or_recycle(model_id, factory).await)
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get binding statistics
    pub fn get_stats(&self) -> BindingStats {
        BindingStats {
            active_bindings: self.viewmodels.len(),
            pending_views: 0,  // No longer have pending with new architecture
            pending_viewmodels: 0,
        }
    }

    /// List all registered Views
    pub fn list_views(&self) -> Vec<ViewId> {
        self.views.keys().copied().collect()
    }

    /// List all bound ViewModels
    pub fn list_viewmodels(&self) -> Vec<ViewId> {
        self.viewmodels.keys().copied().collect()
    }
}

impl Default for BindingRegistry {
    fn default() -> Self {
        Self::new()
    }
}
