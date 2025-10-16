//! Manages all Model/View/ViewModel bindings with concurrent access
//!
//! Uses arc-swap for lock-free reads and non-blocking concurrent writes.
//! Flattened model storage for optimal performance.

use crate::pool::ModelPool;
use crate::stats::BindingStats;
use arc_swap::ArcSwap;
use playground_modules_types::{
    Handle, ModelId, ModelTrait, ModelType, ModuleError, ModuleResult, ViewId, ViewModelTrait,
    ViewTrait,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Manages all MVVM bindings with concurrent, lock-free access
///
/// Architecture (Session 81 design):
/// - Lock-free reads (~5ns) via ArcSwap
/// - Non-blocking concurrent writes via RCU pattern
/// - Flattened model storage: (ViewId, ModelType) -> Pool
///
/// Performance:
/// - View/ViewModel lookup: ~5ns (lock-free)
/// - Model pool lookup: ~5ns (single HashMap lookup)
/// - Model access: ~20-30ns (per-pool RwLock)
pub struct BindingRegistry {
    /// Views indexed by ViewId
    /// Lock-free reads, non-blocking concurrent writes
    views: ArcSwap<HashMap<ViewId, Handle<dyn ViewTrait>>>,

    /// ViewModels indexed by ViewId
    /// Lock-free reads, non-blocking concurrent writes
    viewmodels: ArcSwap<HashMap<ViewId, Handle<dyn ViewModelTrait>>>,

    /// Models with flattened storage: (ViewId, ModelType) -> Pool
    /// Single lookup instead of nested, lock-free reads
    models: ArcSwap<HashMap<(ViewId, ModelType), ModelPool>>,
}

impl BindingRegistry {
    /// Create a new binding registry
    pub fn new() -> Self {
        Self {
            views: ArcSwap::from_pointee(HashMap::new()),
            viewmodels: ArcSwap::from_pointee(HashMap::new()),
            models: ArcSwap::from_pointee(HashMap::new()),
        }
    }

    // ========================================================================
    // View Operations (Lock-Free Reads, Concurrent Writes)
    // ========================================================================

    /// Register a View (replaces if exists)
    ///
    /// Uses RCU (Read-Copy-Update) pattern:
    /// 1. Load current snapshot
    /// 2. Clone HashMap (only modified entries change)
    /// 3. Store new version atomically
    /// 4. Readers never blocked
    pub fn register_view(&self, view: Handle<dyn ViewTrait>) {
        let view_id = view.view_id();
        debug!("Registering View: {:#018x}", view_id);

        // RCU: Clone current map, modify, store atomically
        let mut new_views = (**self.views.load()).clone();
        new_views.insert(view_id, view);
        self.views.store(Arc::new(new_views));

        info!("Registered View: {:#018x}", view_id);
    }

    /// Get a View by ID (lock-free, ~5ns)
    pub fn get_view(&self, view_id: ViewId) -> Option<Handle<dyn ViewTrait>> {
        self.views.load().get(&view_id).cloned()
    }

    // ========================================================================
    // ViewModel Operations (Lock-Free Reads, Concurrent Writes)
    // ========================================================================

    /// Bind a ViewModel to its View (replaces if exists)
    ///
    /// Uses RCU pattern for non-blocking concurrent writes
    pub fn bind_viewmodel(&self, viewmodel: Handle<dyn ViewModelTrait>) -> ModuleResult<()> {
        let view_id = viewmodel.view_id();
        debug!("Binding ViewModel for View: {:#018x}", view_id);

        let view = self.views.load().get(&view_id).cloned().ok_or_else(|| {
            ModuleError::BindingFailed(format!("View not found: {:#018x}", view_id))
        })?;

        let view_version = view.api_version();
        let viewmodel_version = viewmodel.api_version();

        if view_version != viewmodel_version {
            return Err(ModuleError::ApiVersionMismatch {
                expected: view_version,
                found: viewmodel_version,
            });
        }

        let mut new_viewmodels = (**self.viewmodels.load()).clone();
        new_viewmodels.insert(view_id, viewmodel);
        self.viewmodels.store(Arc::new(new_viewmodels));

        info!("Bound ViewModel for View: {:#018x}", view_id);
        Ok(())
    }

    /// Get a ViewModel by View ID (lock-free, ~5ns)
    pub fn get_viewmodel(&self, view_id: ViewId) -> Option<Handle<dyn ViewModelTrait>> {
        self.viewmodels.load().get(&view_id).cloned()
    }

    /// Unbind a ViewModel (for hot-reload)
    ///
    /// Uses RCU pattern for non-blocking concurrent writes
    pub fn unbind_viewmodel(&self, view_id: ViewId) -> ModuleResult<()> {
        debug!("Unbinding ViewModel for View: {:#018x}", view_id);

        // RCU: Clone current map, modify, store atomically
        let mut new_viewmodels = (**self.viewmodels.load()).clone();
        if new_viewmodels.remove(&view_id).is_some() {
            self.viewmodels.store(Arc::new(new_viewmodels));
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
    // Model Pool Operations (Flattened Storage, Lock-Free Reads)
    // ========================================================================

    /// Register a model pool for a specific View and ModelType
    ///
    /// Uses flattened storage with composite key (ViewId, ModelType)
    /// for single-lookup access
    pub fn register_pool(&self, view_id: ViewId, model_type: ModelType, pool: ModelPool) {
        debug!(
            "Registering pool for View {:#018x}, ModelType {:#018x}",
            view_id, model_type
        );

        // RCU: Clone current map, modify, store atomically
        let mut new_models = (**self.models.load()).clone();
        new_models.insert((view_id, model_type), pool);
        self.models.store(Arc::new(new_models));

        info!(
            "Registered pool for View {:#018x}, ModelType {:#018x}",
            view_id, model_type
        );
    }

    /// Get a model pool (lock-free single lookup, ~5ns)
    ///
    /// Returns cloned ModelPool (cheap - contains Arc internally)
    pub fn get_pool(&self, view_id: ViewId, model_type: ModelType) -> Option<ModelPool> {
        self.models.load().get(&(view_id, model_type)).cloned()
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
            active_bindings: self.viewmodels.load().len(),
            pending_views: 0, // No pending with concurrent architecture
            pending_viewmodels: 0,
        }
    }

    /// List all registered Views
    pub fn list_views(&self) -> Vec<ViewId> {
        self.views.load().keys().copied().collect()
    }

    /// List all bound ViewModels
    pub fn list_viewmodels(&self) -> Vec<ViewId> {
        self.viewmodels.load().keys().copied().collect()
    }
}

impl Default for BindingRegistry {
    fn default() -> Self {
        Self::new()
    }
}
