//! Storage view trait - API contract only

use async_trait::async_trait;
use crate::{
    EcsResult,
    model::{World, Entity, Storage, StorageId},
};

/// Storage and persistence API contract
#[async_trait]
pub trait StorageView: Send + Sync {
    /// Create a new storage configuration
    async fn create_storage(&self, world: &World, path: String, format: String) -> EcsResult<Storage>;

    /// Save the entire world to storage
    async fn save_world(&self, world: &World, storage: &Storage) -> EcsResult<()>;

    /// Load the entire world from storage
    async fn load_world(&self, world: &World, storage: &Storage) -> EcsResult<()>;

    /// Save specific entities to storage
    async fn save_entities(&self, world: &World, storage: &Storage, entities: Vec<Entity>) -> EcsResult<()>;

    /// Load entities from storage
    async fn load_entities(&self, world: &World, storage: &Storage) -> EcsResult<Vec<Entity>>;

    /// Clear storage contents
    async fn clear_storage(&self, storage: &Storage) -> EcsResult<()>;

    /// Check if storage exists
    async fn storage_exists(&self, storage: &Storage) -> EcsResult<bool>;

    /// Delete storage
    async fn delete_storage(&self, world: &World, storage: &Storage) -> EcsResult<()>;

    /// Get a storage by ID
    async fn get_storage(&self, world: &World, storage_id: StorageId) -> EcsResult<Storage>;

    /// Get all storages
    async fn get_all_storages(&self, world: &World) -> EcsResult<Vec<Storage>>;

    /// Create a snapshot of current world state
    async fn create_snapshot(&self, world: &World, name: String) -> EcsResult<StorageId>;

    /// Restore world from a snapshot
    async fn restore_snapshot(&self, world: &World, snapshot_id: StorageId) -> EcsResult<()>;

    /// List all snapshots
    async fn list_snapshots(&self, world: &World) -> EcsResult<Vec<(StorageId, String)>>;

    /// Delete a snapshot
    async fn delete_snapshot(&self, world: &World, snapshot_id: StorageId) -> EcsResult<()>;

    /// Export world to JSON format
    async fn export_json(&self, world: &World, path: String) -> EcsResult<()>;

    /// Import world from JSON format
    async fn import_json(&self, world: &World, path: String) -> EcsResult<()>;

    /// Get storage size in bytes
    async fn get_storage_size(&self, storage: &Storage) -> EcsResult<usize>;
}