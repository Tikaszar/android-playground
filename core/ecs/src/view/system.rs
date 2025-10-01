//! System Management API functions
//!
//! This module provides the View layer API contracts for system registration and execution.
//! These functions are stubs that will be replaced by the actual implementations
//! from systems/ecs at compile time through conditional compilation.

use crate::{
    EcsResult, EcsError,
    model::{World, System, SystemId, QueryId},
};

/// Register a new system
pub async fn register_system(
    _world: &World,
    _name: String,
    _query: QueryId,
    _dependencies: Vec<SystemId>
) -> EcsResult<System> {
    Err(EcsError::ModuleNotFound("register_system not implemented - systems/ecs required".to_string()))
}

/// Unregister a system
pub async fn unregister_system(_world: &World, _system: &System) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("unregister_system not implemented - systems/ecs required".to_string()))
}

/// Run a single system
pub async fn run_system(_world: &World, _system: &System) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("run_system not implemented - systems/ecs required".to_string()))
}

/// Run multiple systems
pub async fn run_systems(_world: &World, _systems: Vec<System>) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("run_systems not implemented - systems/ecs required".to_string()))
}

/// Schedule systems based on dependencies
/// Returns the execution order
pub async fn schedule_systems(_world: &World) -> EcsResult<Vec<System>> {
    Err(EcsError::ModuleNotFound("schedule_systems not implemented - systems/ecs required".to_string()))
}

/// Get a system by ID
pub async fn get_system(_world: &World, _system_id: SystemId) -> EcsResult<System> {
    Err(EcsError::ModuleNotFound("get_system not implemented - systems/ecs required".to_string()))
}

/// Get all registered systems
pub async fn get_all_systems(_world: &World) -> EcsResult<Vec<System>> {
    Err(EcsError::ModuleNotFound("get_all_systems not implemented - systems/ecs required".to_string()))
}

/// Enable a system
pub async fn enable_system(_world: &World, _system_id: SystemId) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("enable_system not implemented - systems/ecs required".to_string()))
}

/// Disable a system
pub async fn disable_system(_world: &World, _system_id: SystemId) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("disable_system not implemented - systems/ecs required".to_string()))
}

/// Check if a system is enabled
pub async fn is_system_enabled(_world: &World, _system_id: SystemId) -> EcsResult<bool> {
    Err(EcsError::ModuleNotFound("is_system_enabled not implemented - systems/ecs required".to_string()))
}

/// Execute all systems in dependency order
pub async fn step_systems(_world: &World, _delta_time: f32) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("step_systems not implemented - systems/ecs required".to_string()))
}

/// Get system execution statistics
pub async fn get_system_stats(_world: &World, _system_id: SystemId) -> EcsResult<SystemStats> {
    Err(EcsError::ModuleNotFound("get_system_stats not implemented - systems/ecs required".to_string()))
}

/// Statistics for system execution
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub execution_count: u64,
    pub total_time_ms: f64,
    pub average_time_ms: f64,
    pub last_execution_time_ms: f64,
}

/// Get system dependencies
pub async fn get_system_dependencies(_world: &World, _system_id: SystemId) -> EcsResult<Vec<SystemId>> {
    Err(EcsError::ModuleNotFound("get_system_dependencies not implemented - systems/ecs required".to_string()))
}

/// Update system dependencies
pub async fn update_system_dependencies(_world: &World, _system_id: SystemId, _dependencies: Vec<SystemId>) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("update_system_dependencies not implemented - systems/ecs required".to_string()))
}

/// Get systems that depend on a given system
pub async fn get_dependent_systems(_world: &World, _system_id: SystemId) -> EcsResult<Vec<SystemId>> {
    Err(EcsError::ModuleNotFound("get_dependent_systems not implemented - systems/ecs required".to_string()))
}

/// Check if system exists
pub async fn system_exists(_world: &World, _system_id: SystemId) -> EcsResult<bool> {
    Err(EcsError::ModuleNotFound("system_exists not implemented - systems/ecs required".to_string()))
}

/// Clear all system statistics
pub async fn clear_system_stats(_world: &World) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("clear_system_stats not implemented - systems/ecs required".to_string()))
}