//! System execution for the ECS
//! 
//! Systems are concrete structs that process entities and components.

use tokio::sync::mpsc;
use playground_core_types::Handle;
use crate::{World, CoreResult, CoreError};

/// Execution stage for systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionStage {
    /// Pre-update stage
    PreUpdate,
    /// Main update stage
    Update,
    /// Post-update stage
    PostUpdate,
    /// Layout calculation stage
    Layout,
    /// Rendering stage
    Render,
}

/// Command to update a system
pub struct SystemUpdateCommand {
    pub delta_time: f32,
    pub world: Handle<World>,
    pub response: tokio::sync::oneshot::Sender<CoreResult<()>>,
}

/// Concrete system struct (not a trait!)
pub struct System {
    /// System name
    pub name: String,
    
    /// Execution stage
    pub stage: ExecutionStage,
    
    /// Dependencies (other systems that must run first)
    pub dependencies: Vec<String>,
    
    /// Whether the system is enabled
    pub enabled: bool,
    
    /// Channel to send update commands to the actual implementation
    update_sender: mpsc::Sender<SystemUpdateCommand>,
}

impl System {
    /// Create a new system
    pub fn new(
        name: String,
        stage: ExecutionStage,
        update_sender: mpsc::Sender<SystemUpdateCommand>,
    ) -> Self {
        Self {
            name,
            stage,
            dependencies: Vec::new(),
            enabled: true,
            update_sender,
        }
    }
    
    /// Set dependencies (builder pattern)
    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }
    
    /// Enable the system
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Disable the system
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Update the system
    pub async fn update(&self, delta_time: f32, world: Handle<World>) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.update_sender.send(SystemUpdateCommand {
            delta_time,
            world,
            response: tx,
        }).await.map_err(|_| CoreError::SendError)?;
        
        rx.await.map_err(|_| CoreError::ReceiveError)?
    }
}

/// System scheduler for managing execution order
pub struct SystemScheduler {
    /// Systems organized by stage
    systems: Vec<System>,
}

impl SystemScheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }
    
    /// Add a system to the scheduler
    pub fn add_system(&mut self, system: System) {
        self.systems.push(system);
        self.sort_systems();
    }
    
    /// Remove a system by name
    pub fn remove_system(&mut self, name: &str) {
        self.systems.retain(|s| s.name != name);
    }
    
    /// Get a system by name
    pub fn get_system(&mut self, name: &str) -> Option<&mut System> {
        self.systems.iter_mut().find(|s| s.name == name)
    }
    
    /// Sort systems by stage and dependencies
    fn sort_systems(&mut self) {
        self.systems.sort_by(|a, b| {
            // First sort by stage
            match a.stage.cmp(&b.stage) {
                std::cmp::Ordering::Equal => {
                    // Then by dependencies
                    if b.dependencies.contains(&a.name) {
                        std::cmp::Ordering::Less
                    } else if a.dependencies.contains(&b.name) {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                other => other,
            }
        });
    }
    
    /// Execute all systems for a frame
    pub async fn execute(&self, delta_time: f32, world: Handle<World>) -> CoreResult<()> {
        for system in &self.systems {
            if system.enabled {
                system.update(delta_time, world.clone()).await?;
            }
        }
        Ok(())
    }
    
    /// Execute systems for a specific stage
    pub async fn execute_stage(
        &self, 
        stage: ExecutionStage, 
        delta_time: f32, 
        world: Handle<World>
    ) -> CoreResult<()> {
        for system in &self.systems {
            if system.enabled && system.stage == stage {
                system.update(delta_time, world.clone()).await?;
            }
        }
        Ok(())
    }
    
    /// Get all systems in a stage
    pub fn get_stage_systems(&self, stage: ExecutionStage) -> Vec<&System> {
        self.systems
            .iter()
            .filter(|s| s.stage == stage)
            .collect()
    }
    
    /// Enable all systems
    pub fn enable_all(&mut self) {
        for system in &mut self.systems {
            system.enabled = true;
        }
    }
    
    /// Disable all systems
    pub fn disable_all(&mut self) {
        for system in &mut self.systems {
            system.enabled = false;
        }
    }
}