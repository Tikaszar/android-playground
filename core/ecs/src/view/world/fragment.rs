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

    async fn reset_world(&self, _world: &World) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clear_world(&self, _world: &World) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn save_world_state(&self, _world: &World) -> EcsResult<Bytes> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn load_world_state(&self, _world: &World, _state: Bytes) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_world_stats(&self, _world: &World) -> EcsResult<WorldStats> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_world_metadata(&self, _world: &World) -> EcsResult<WorldMetadata> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn set_world_metadata(&self, _world: &World, _metadata: WorldMetadata) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_entity_count(&self, _world: &World) -> EcsResult<u32> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_component_count(&self, _world: &World) -> EcsResult<u32> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_system_count(&self, _world: &World) -> EcsResult<u32> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }
}