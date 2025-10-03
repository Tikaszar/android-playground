//! Storage view stub implementation

use async_trait::async_trait;
use playground_modules_types::{ViewFragmentTrait, ViewId, FragmentId};
use crate::{
    EcsResult, EcsError,
    model::{World, Entity, Storage, StorageId},
    view::storage::StorageView,
};

pub const STORAGE_FRAGMENT_ID: FragmentId = 0x0005;

/// EcsView implementation for StorageView fragment
pub struct EcsView;

#[async_trait]
impl ViewFragmentTrait for EcsView {
    fn view_id(&self) -> ViewId {
        crate::ECS_VIEW_ID
    }

    fn fragment_id(&self) -> FragmentId {
        STORAGE_FRAGMENT_ID
    }
}

#[async_trait]
impl StorageView for EcsView {
    async fn create_storage(&self, _world: &World, _path: String, _format: String) -> EcsResult<Storage> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn save_world(&self, _world: &World, _storage: &Storage) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn load_world(&self, _world: &World, _storage: &Storage) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn save_entities(&self, _world: &World, _storage: &Storage, _entities: Vec<Entity>) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn load_entities(&self, _world: &World, _storage: &Storage) -> EcsResult<Vec<Entity>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clear_storage(&self, _storage: &Storage) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn storage_exists(&self, _storage: &Storage) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn delete_storage(&self, _world: &World, _storage: &Storage) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_storage(&self, _world: &World, _storage_id: StorageId) -> EcsResult<Storage> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_all_storages(&self, _world: &World) -> EcsResult<Vec<Storage>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn create_snapshot(&self, _world: &World, _name: String) -> EcsResult<StorageId> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn restore_snapshot(&self, _world: &World, _snapshot_id: StorageId) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn list_snapshots(&self, _world: &World) -> EcsResult<Vec<(StorageId, String)>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn delete_snapshot(&self, _world: &World, _snapshot_id: StorageId) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn export_json(&self, _world: &World, _path: String) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn import_json(&self, _world: &World, _path: String) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_storage_size(&self, _storage: &Storage) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }
}