//! Storage System API functions
//!
//! This module provides the View layer API contracts for storage/persistence operations.
//! These functions are stubs that will be replaced by the actual implementations
//! from systems/ecs at compile time through conditional compilation.

use crate::{
    EcsResult, EcsError,
    model::{World, Entity, Storage, StorageId},
};

/// Create a new storage configuration
pub async fn create_storage(_world: &World, _path: String, _format: String) -> EcsResult<Storage> {
    Err(EcsError::ModuleNotFound("create_storage not implemented - systems/ecs required".to_string()))
}

/// Save the entire world to storage
pub async fn save_world(_world: &World, _storage: &Storage) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("save_world not implemented - systems/ecs required".to_string()))
}

/// Load the entire world from storage
pub async fn load_world(_world: &World, _storage: &Storage) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("load_world not implemented - systems/ecs required".to_string()))
}

/// Save specific entities to storage
pub async fn save_entities(_world: &World, _storage: &Storage, _entities: Vec<Entity>) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("save_entities not implemented - systems/ecs required".to_string()))
}

/// Load entities from storage
pub async fn load_entities(_world: &World, _storage: &Storage) -> EcsResult<Vec<Entity>> {
    Err(EcsError::ModuleNotFound("load_entities not implemented - systems/ecs required".to_string()))
}

/// Clear storage contents
pub async fn clear_storage(_storage: &Storage) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("clear_storage not implemented - systems/ecs required".to_string()))
}

/// Check if storage exists
pub async fn storage_exists(_storage: &Storage) -> EcsResult<bool> {
    Err(EcsError::ModuleNotFound("storage_exists not implemented - systems/ecs required".to_string()))
}

/// Delete storage
pub async fn delete_storage(_world: &World, _storage: &Storage) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("delete_storage not implemented - systems/ecs required".to_string()))
}

/// Get a storage by ID
pub async fn get_storage(_world: &World, _storage_id: StorageId) -> EcsResult<Storage> {
    Err(EcsError::ModuleNotFound("get_storage not implemented - systems/ecs required".to_string()))
}

/// Get all storages
pub async fn get_all_storages(_world: &World) -> EcsResult<Vec<Storage>> {
    Err(EcsError::ModuleNotFound("get_all_storages not implemented - systems/ecs required".to_string()))
}

/// Create a snapshot of current world state
pub async fn create_snapshot(_world: &World, _name: String) -> EcsResult<StorageId> {
    Err(EcsError::ModuleNotFound("create_snapshot not implemented - systems/ecs required".to_string()))
}

/// Restore world from a snapshot
pub async fn restore_snapshot(_world: &World, _snapshot_id: StorageId) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("restore_snapshot not implemented - systems/ecs required".to_string()))
}

/// List all snapshots
pub async fn list_snapshots(_world: &World) -> EcsResult<Vec<(StorageId, String)>> {
    Err(EcsError::ModuleNotFound("list_snapshots not implemented - systems/ecs required".to_string()))
}

/// Delete a snapshot
pub async fn delete_snapshot(_world: &World, _snapshot_id: StorageId) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("delete_snapshot not implemented - systems/ecs required".to_string()))
}

/// Export world to JSON format
pub async fn export_json(_world: &World, _path: String) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("export_json not implemented - systems/ecs required".to_string()))
}

/// Import world from JSON format
pub async fn import_json(_world: &World, _path: String) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("import_json not implemented - systems/ecs required".to_string()))
}

/// Get storage size in bytes
pub async fn get_storage_size(_storage: &Storage) -> EcsResult<usize> {
    Err(EcsError::ModuleNotFound("get_storage_size not implemented - systems/ecs required".to_string()))
}