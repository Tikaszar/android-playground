//! System fragment implementation

use async_trait::async_trait;
use crate::{
    EcsResult, EcsError,
    model::{World, System, SystemId, SystemStats, QueryId},
    view::system::SystemView,
};

/// System operations fragment
pub struct SystemFragment;

#[async_trait]
impl SystemView for SystemFragment {
    async fn register_system(&self, _world: &World, _name: String, _query: QueryId, _dependencies: Vec<SystemId>) -> EcsResult<System> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn unregister_system(&self, _world: &World, _system: &System) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn run_system(&self, _world: &World, _system: &System) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn run_systems(&self, _world: &World, _systems: Vec<System>) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn schedule_systems(&self, _world: &World) -> EcsResult<Vec<System>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_system(&self, _world: &World, _system_id: SystemId) -> EcsResult<System> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_all_systems(&self, _world: &World) -> EcsResult<Vec<System>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn enable_system(&self, _world: &World, _system_id: SystemId) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn disable_system(&self, _world: &World, _system_id: SystemId) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn is_system_enabled(&self, _world: &World, _system_id: SystemId) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn step_systems(&self, _world: &World, _delta_time: f32) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_system_stats(&self, _world: &World, _system_id: SystemId) -> EcsResult<SystemStats> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_system_dependencies(&self, _world: &World, _system_id: SystemId) -> EcsResult<Vec<SystemId>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn update_system_dependencies(&self, _world: &World, _system_id: SystemId, _dependencies: Vec<SystemId>) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_dependent_systems(&self, _world: &World, _system_id: SystemId) -> EcsResult<Vec<SystemId>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn system_exists(&self, _world: &World, _system_id: SystemId) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clear_system_stats(&self, _world: &World) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }
}