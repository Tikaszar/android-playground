//! World view trait - API contract only

use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;
use crate::{
    EcsResult,
    model::{World, WorldStats, WorldMetadata},
};

/// World management API contract
#[async_trait]
pub trait WorldView: Send + Sync {
    /// Initialize the world
    async fn initialize_world(&self) -> EcsResult<Arc<World>>;

    /// Get the world instance
    async fn get_world(&self) -> EcsResult<Arc<World>>;

    /// Shutdown the world
    async fn shutdown_world(&self) -> EcsResult<()>;

    /// Clear all entities and components
    async fn clear_world(&self, world: &World) -> EcsResult<()>;

    /// Step the world forward by delta time
    async fn step(&self, world: &World, delta_time: f32) -> EcsResult<()>;

    /// Get world statistics
    async fn get_stats(&self, world: &World) -> EcsResult<WorldStats>;

    /// Merge another world into this one
    async fn merge_worlds(&self, target: &World, source: &World) -> EcsResult<()>;

    /// Clone the world
    async fn clone_world(&self, world: &World) -> EcsResult<Arc<World>>;

    // Resource management (global singletons)

    /// Insert a resource into the world
    async fn insert_resource(&self, world: &World, type_name: String, data: Bytes) -> EcsResult<()>;

    /// Get a resource from the world
    async fn get_resource(&self, world: &World, type_name: String) -> EcsResult<Bytes>;

    /// Remove a resource from the world
    async fn remove_resource(&self, world: &World, type_name: String) -> EcsResult<()>;

    /// Check if a resource exists
    async fn has_resource(&self, world: &World, type_name: String) -> EcsResult<bool>;

    /// Get all resource type names
    async fn get_all_resources(&self, world: &World) -> EcsResult<Vec<String>>;

    /// Reset the world to initial state (keep structure, clear data)
    async fn reset_world(&self, world: &World) -> EcsResult<()>;

    /// Lock the world for exclusive access
    async fn lock_world(&self, world: &World) -> EcsResult<()>;

    /// Unlock the world
    async fn unlock_world(&self, world: &World) -> EcsResult<()>;

    /// Check if world is locked
    async fn is_world_locked(&self, world: &World) -> EcsResult<bool>;

    /// Validate world integrity (check for dangling references, etc)
    async fn validate_world(&self, world: &World) -> EcsResult<Vec<String>>;

    /// Get world metadata (creation time, last modified, etc)
    async fn get_world_metadata(&self, world: &World) -> EcsResult<WorldMetadata>;
}