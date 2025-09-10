//! System contracts for the ECS
//! 
//! This defines the contract for systems that run in the ECS.

use async_trait::async_trait;
use crate::EcsResult;

/// Execution stage for systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionStage {
    /// Game logic, input handling, state changes
    Update,
    /// UI layout calculations, spatial organization  
    Layout,
    /// Generate render commands for browser
    Render,
}

impl ExecutionStage {
    /// Get all stages in execution order
    pub fn all() -> &'static [ExecutionStage] {
        &[
            ExecutionStage::Update,
            ExecutionStage::Layout,
            ExecutionStage::Render,
        ]
    }
}

/// System trait for all systems (engine systems and plugins)
/// 
/// Systems are the logic units that operate on entities and components.
/// They run in specific execution stages and can be plugins or engine systems.
#[async_trait]
pub trait System: Send + Sync {
    /// Get the system name
    fn name(&self) -> &str;
    
    /// Get the execution stage for this system
    fn stage(&self) -> ExecutionStage;
    
    /// Initialize the system
    async fn initialize(&mut self) -> EcsResult<()> {
        Ok(())
    }
    
    /// Update the system
    async fn update(&mut self, delta_time: f32) -> EcsResult<()>;
    
    /// Cleanup the system
    async fn cleanup(&mut self) -> EcsResult<()> {
        Ok(())
    }
}