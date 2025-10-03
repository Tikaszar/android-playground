//! World fragment implementation

use async_trait::async_trait;
use std::sync::Arc;
use bytes::Bytes;
use crate::{
    EcsResult, EcsError,
    model::{World, WorldStats, WorldMetadata},
    view::world::WorldView,
};

/// World operations fragment
pub struct WorldFragment;

#[async_trait]
impl WorldView for WorldFragment {
    async fn initialize_world(&self) -> EcsResult<Arc<World>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_world(&self) -> EcsResult<Arc<World>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn shutdown_world(&self) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clear_world(&self, _world: &World) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn step(&self, _world: &World, _delta_time: f32) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_stats(&self, _world: &World) -> EcsResult<WorldStats> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn merge_worlds(&self, _target: &World, _source: &World) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clone_world(&self, _world: &World) -> EcsResult<Arc<World>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn insert_resource(&self, _world: &World, _type_name: String, _data: Bytes) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_resource(&self, _world: &World, _type_name: String) -> EcsResult<Bytes> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn remove_resource(&self, _world: &World, _type_name: String) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn has_resource(&self, _world: &World, _type_name: String) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_all_resources(&self, _world: &World) -> EcsResult<Vec<String>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn reset_world(&self, _world: &World) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn lock_world(&self, _world: &World) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn unlock_world(&self, _world: &World) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn is_world_locked(&self, _world: &World) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn validate_world(&self, _world: &World) -> EcsResult<Vec<String>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_world_metadata(&self, _world: &World) -> EcsResult<WorldMetadata> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_entity_count(&self, _world: &World) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }
}