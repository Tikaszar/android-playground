//! Model pool with recycling for efficient memory management

use playground_modules_types::{Handle, ModelId, ModelTrait, Shared};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Pool for a specific Model type with object recycling
///
/// Provides:
/// - Concurrent access via RwLock at pool level (fine-grained locking)
/// - Object recycling to reduce allocations
/// - Thread-safe operations
#[derive(Clone)]
pub struct ModelPool {
    /// Active models currently in use
    active: Shared<HashMap<ModelId, Handle<dyn ModelTrait>>>,

    /// Recycled models ready for reuse (reduces allocations)
    recycled: Shared<Vec<Handle<dyn ModelTrait>>>,
}

impl ModelPool {
    /// Create a new empty pool
    pub fn new() -> Self {
        Self {
            active: Handle::new(RwLock::new(HashMap::new())),
            recycled: Handle::new(RwLock::new(Vec::new())),
        }
    }

    /// Get a model by ID (read lock only)
    pub async fn get(&self, model_id: ModelId) -> Option<Handle<dyn ModelTrait>> {
        let active = self.active.read().await;
        active.get(&model_id).cloned()
    }

    /// Insert a model into the pool (write lock)
    pub async fn insert(&self, model_id: ModelId, model: Handle<dyn ModelTrait>) {
        let mut active = self.active.write().await;
        active.insert(model_id, model);
    }

    /// Remove a model and add to recycle pool for reuse
    pub async fn remove(&self, model_id: ModelId) -> Option<Handle<dyn ModelTrait>> {
        let mut active = self.active.write().await;
        if let Some(model) = active.remove(&model_id) {
            // Add to recycle pool for later reuse
            let mut recycled = self.recycled.write().await;
            recycled.push(model.clone());
            Some(model)
        } else {
            None
        }
    }

    /// Get or create a model, preferring recycled objects
    ///
    /// This reduces allocations by reusing previously deleted models
    pub async fn get_or_recycle<F>(&self, model_id: ModelId, factory: F) -> Handle<dyn ModelTrait>
    where
        F: FnOnce() -> Handle<dyn ModelTrait>,
    {
        // Try to get from recycle pool first
        let mut recycled = self.recycled.write().await;
        let model = if let Some(recycled_model) = recycled.pop() {
            // Reuse recycled model - no allocation!
            recycled_model
        } else {
            // No recycled models available, create new
            drop(recycled);
            factory()
        };

        // Insert into active pool
        let mut active = self.active.write().await;
        active.insert(model_id, model.clone());
        model
    }

    /// Get count of active models
    pub async fn active_count(&self) -> usize {
        let active = self.active.read().await;
        active.len()
    }

    /// Get count of recycled models ready for reuse
    pub async fn recycled_count(&self) -> usize {
        let recycled = self.recycled.read().await;
        recycled.len()
    }

    /// Clear all recycled models (release memory)
    pub async fn clear_recycled(&self) {
        let mut recycled = self.recycled.write().await;
        recycled.clear();
    }
}

impl Default for ModelPool {
    fn default() -> Self {
        Self::new()
    }
}
