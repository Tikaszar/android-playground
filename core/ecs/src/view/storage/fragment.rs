//! Storage fragment implementation

use async_trait::async_trait;
use crate::{
    EcsResult, EcsError,
    model::{World, Entity, Storage, StorageId},
    view::storage::StorageView,
};

/// Storage operations fragment
pub struct StorageFragment;

#[async_trait]
impl StorageView for StorageFragment {
    async fn create_storage(&self, _world: &World, _path: String, _format: String) -> EcsResult<Storage> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn delete_storage(&self, _world: &World, _storage_id: StorageId) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_storage(&self, _world: &World, _storage_id: StorageId) -> EcsResult<Storage> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn storage_exists(&self, _world: &World, _storage_id: StorageId) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_all_storages(&self, _world: &World) -> EcsResult<Vec<Storage>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn save_world(&self, _world: &World, _storage_id: StorageId) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn load_world(&self, _world: &World, _storage_id: StorageId) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn save_entities(&self, _world: &World, _storage_id: StorageId, _entities: Vec<Entity>) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn load_entities(&self, _world: &World, _storage_id: StorageId) -> EcsResult<Vec<Entity>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clear_storage(&self, _world: &World, _storage_id: StorageId) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_storage_size(&self, _world: &World, _storage_id: StorageId) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn create_snapshot(&self, _world: &World, _storage_id: StorageId, _name: String) -> EcsResult<String> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn restore_snapshot(&self, _world: &World, _storage_id: StorageId, _snapshot_id: String) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn delete_snapshot(&self, _world: &World, _storage_id: StorageId, _snapshot_id: String) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn list_snapshots(&self, _world: &World, _storage_id: StorageId) -> EcsResult<Vec<String>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn export_json(&self, _world: &World, _storage_id: StorageId) -> EcsResult<String> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn import_json(&self, _world: &World, _storage_id: StorageId, _json: String) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }
}