//! System scheduler for staged execution in the unified ECS

use std::collections::HashMap;
use async_trait::async_trait;
use playground_core_types::{Shared, shared};
use playground_core_ecs::{EcsResult, System, ExecutionStage};
use crate::world::World;

// Re-export the System trait from core
pub use playground_core_ecs::System as SystemTrait;

/// System registration info
struct SystemInfo {
    name: String,
    stage: ExecutionStage,
    priority: i32,
}

/// System scheduler manages system execution in stages
pub struct SystemScheduler {
    // Systems organized by stage
    // Using Vec instead of Box<dyn System> to avoid dyn
    // Systems will be registered through a concrete wrapper type
    systems_by_stage: Shared<HashMap<ExecutionStage, Vec<SystemHandle>>>,
    system_info: Shared<HashMap<String, SystemInfo>>,
}

/// Handle to a system (avoiding dyn trait objects)
/// This will be extended with concrete system types
pub struct SystemHandle {
    name: String,
    stage: ExecutionStage,
    // For now, we'll use channels to communicate with systems
    // This avoids the need for dyn trait objects
    update_sender: Option<tokio::sync::mpsc::UnboundedSender<(f32,)>>,
}

impl SystemScheduler {
    /// Create a new system scheduler
    pub fn new() -> Self {
        let mut systems_by_stage = HashMap::new();
        
        // Initialize with empty vecs for each stage
        for stage in ExecutionStage::all() {
            systems_by_stage.insert(*stage, Vec::new());
        }
        
        Self {
            systems_by_stage: shared(systems_by_stage),
            system_info: shared(HashMap::new()),
        }
    }
    
    /// Register a system with the scheduler
    /// Note: This needs to be refactored to avoid dyn
    /// For now, we'll use a channel-based approach
    pub async fn register_system(
        &self,
        name: String,
        stage: ExecutionStage,
        priority: i32,
    ) -> EcsResult<()> {
        let info = SystemInfo {
            name: name.clone(),
            stage,
            priority,
        };
        
        self.system_info.write().await.insert(name.clone(), info);
        
        let handle = SystemHandle {
            name: name.clone(),
            stage,
            update_sender: None,
        };
        
        let mut systems = self.systems_by_stage.write().await;
        if let Some(stage_systems) = systems.get_mut(&stage) {
            stage_systems.push(handle);
            // Sort by priority
            stage_systems.sort_by_key(|_h| {
                // Get priority from info (would need to store it in handle)
                0 // Placeholder
            });
        }
        
        Ok(())
    }
    
    /// Execute all systems in a specific stage
    pub async fn execute_stage(
        &self,
        stage: ExecutionStage,
        _world: &World,
        delta_time: f32,
    ) -> EcsResult<()> {
        let systems = self.systems_by_stage.read().await;
        
        if let Some(stage_systems) = systems.get(&stage) {
            // Execute systems in this stage
            // For now, sequential execution
            // TODO: Implement parallel execution where possible
            for system_handle in stage_systems {
                // Send update message to system through channel
                if let Some(sender) = &system_handle.update_sender {
                    let _ = sender.send((delta_time,));
                }
            }
        }
        
        Ok(())
    }
    
    /// Execute all stages in order
    pub async fn execute_all(
        &self,
        world: &World,
        delta_time: f32,
    ) -> EcsResult<()> {
        for stage in ExecutionStage::all() {
            self.execute_stage(*stage, world, delta_time).await?;
        }
        Ok(())
    }
    
    /// Get registered system count
    pub async fn system_count(&self) -> usize {
        self.system_info.read().await.len()
    }
    
    /// Get systems for a specific stage
    pub async fn get_stage_systems(&self, stage: ExecutionStage) -> Vec<String> {
        let systems = self.systems_by_stage.read().await;
        systems.get(&stage)
            .map(|s| s.iter().map(|h| h.name.clone()).collect())
            .unwrap_or_default()
    }
    
    /// Clear all registered systems
    pub async fn clear(&self) {
        self.systems_by_stage.write().await.clear();
        self.system_info.write().await.clear();
        
        // Re-initialize with empty vecs for each stage
        let mut systems_by_stage = self.systems_by_stage.write().await;
        for stage in ExecutionStage::all() {
            systems_by_stage.insert(*stage, Vec::new());
        }
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Concrete system wrapper to avoid dyn trait objects
/// This will be used to wrap actual system implementations
pub struct ConcreteSystem<T> {
    inner: T,
}

impl<T> ConcreteSystem<T> {
    pub fn new(system: T) -> Self {
        Self { inner: system }
    }
}