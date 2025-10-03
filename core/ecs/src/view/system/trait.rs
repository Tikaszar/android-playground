//! System view trait - API contract only

use async_trait::async_trait;
use crate::{
    EcsResult,
    model::{World, System, SystemId, SystemStats, QueryId},
};

/// System management API contract
#[async_trait]
pub trait SystemView: Send + Sync {
    /// Register a new system
    async fn register_system(&self, world: &World, name: String, query: QueryId, dependencies: Vec<SystemId>) -> EcsResult<System>;

    /// Unregister a system
    async fn unregister_system(&self, world: &World, system: &System) -> EcsResult<()>;

    /// Run a single system
    async fn run_system(&self, world: &World, system: &System) -> EcsResult<()>;

    /// Run multiple systems
    async fn run_systems(&self, world: &World, systems: Vec<System>) -> EcsResult<()>;

    /// Schedule systems based on dependencies
    /// Returns the execution order
    async fn schedule_systems(&self, world: &World) -> EcsResult<Vec<System>>;

    /// Get a system by ID
    async fn get_system(&self, world: &World, system_id: SystemId) -> EcsResult<System>;

    /// Get all registered systems
    async fn get_all_systems(&self, world: &World) -> EcsResult<Vec<System>>;

    /// Enable a system
    async fn enable_system(&self, world: &World, system_id: SystemId) -> EcsResult<()>;

    /// Disable a system
    async fn disable_system(&self, world: &World, system_id: SystemId) -> EcsResult<()>;

    /// Check if a system is enabled
    async fn is_system_enabled(&self, world: &World, system_id: SystemId) -> EcsResult<bool>;

    /// Execute all systems in dependency order
    async fn step_systems(&self, world: &World, delta_time: f32) -> EcsResult<()>;

    /// Get system execution statistics
    async fn get_system_stats(&self, world: &World, system_id: SystemId) -> EcsResult<SystemStats>;

    /// Get system dependencies
    async fn get_system_dependencies(&self, world: &World, system_id: SystemId) -> EcsResult<Vec<SystemId>>;

    /// Update system dependencies
    async fn update_system_dependencies(&self, world: &World, system_id: SystemId, dependencies: Vec<SystemId>) -> EcsResult<()>;

    /// Get systems that depend on a given system
    async fn get_dependent_systems(&self, world: &World, system_id: SystemId) -> EcsResult<Vec<SystemId>>;

    /// Check if system exists
    async fn system_exists(&self, world: &World, system_id: SystemId) -> EcsResult<bool>;

    /// Clear all system statistics
    async fn clear_system_stats(&self, world: &World) -> EcsResult<()>;
}