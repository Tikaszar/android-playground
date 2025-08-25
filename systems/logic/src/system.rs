use crate::error::LogicResult;
use crate::world::World;
use crate::system_data::{SystemData, SystemId};
use playground_core_types::{Shared, shared};
use tokio::task::JoinHandle;
use std::collections::VecDeque;

/// System trait that all game systems must implement
#[async_trait::async_trait]
pub trait System: Send + Sync + 'static {
    /// System name for debugging
    fn name(&self) -> &'static str;
    
    /// System dependencies - systems that must run before this one
    fn dependencies(&self) -> Vec<SystemId> {
        Vec::new()
    }
    
    /// Initialize the system
    async fn initialize(&mut self, world: &World) -> LogicResult<()> {
        Ok(())
    }
    
    /// Run the system
    async fn run(&mut self, world: &World, delta_time: f32) -> LogicResult<()>;
    
    /// Cleanup when system is removed
    async fn cleanup(&mut self, world: &World) -> LogicResult<()> {
        Ok(())
    }
    
    /// Whether this system should run in parallel with others
    fn parallel(&self) -> bool {
        true
    }
}

/// Extension methods for System trait
pub trait SystemExt {
    /// Get the system ID as a string
    fn system_id(&self) -> SystemId;
    /// Get dependencies as strings
    fn dependencies_as_strings(&self) -> Vec<SystemId>;
}

impl<S: System> SystemExt for S {
    fn system_id(&self) -> SystemId {
        format!("{}_{}", std::any::type_name::<S>(), self.name())
    }
    
    fn dependencies_as_strings(&self) -> Vec<SystemId> {
        self.dependencies()
    }
}

/// System metadata for runtime management
pub struct SystemInfo {
    pub system_id: SystemId,
    pub name: String,
    pub dependencies: Vec<SystemId>,
    pub parallel: bool,
    pub retry_count: u32,
    pub max_retries: u32,
    pub enabled: bool,
    pub safe_mode: bool,
}

/// System instance wrapper using concrete SystemData
pub struct SystemInstance {
    pub info: SystemInfo,
    pub system_data: SystemData,
    pub last_error: Option<String>,
}

/// Stage in the system execution pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    PreUpdate,
    Update,
    PostUpdate,
    PreRender,
    Render,
    PostRender,
}

/// System registration builder
pub struct SystemRegistration {
    system_data: SystemData,
    stage: Stage,
    dependencies: Vec<SystemId>,
    before: Vec<SystemId>,
    after: Vec<SystemId>,
    system_id: SystemId,
    name: String,
    parallel: bool,
}

impl SystemRegistration {
    pub fn new<S: System + SystemExt>(system: S) -> Self {
        let system_id = system.system_id();
        let name = system.name().to_string();
        let dependencies = system.dependencies();
        let parallel = system.parallel();
        let system_data = SystemData::new(system);
        
        Self {
            system_data,
            stage: Stage::Update,
            dependencies,
            before: Vec::new(),
            after: Vec::new(),
            system_id,
            name,
            parallel,
        }
    }
    
    pub fn stage(mut self, stage: Stage) -> Self {
        self.stage = stage;
        self
    }
    
    pub fn depends_on(mut self, system_id: SystemId) -> Self {
        self.dependencies.push(system_id);
        self
    }
    
    pub fn runs_before(mut self, system_id: SystemId) -> Self {
        self.before.push(system_id);
        self
    }
    
    pub fn runs_after(mut self, system_id: SystemId) -> Self {
        self.after.push(system_id);
        self
    }
    
    pub fn build(self) -> (Stage, SystemInstance) {
        let info = SystemInfo {
            system_id: self.system_id,
            name: self.name,
            dependencies: self.dependencies,
            parallel: self.parallel,
            retry_count: 0,
            max_retries: 3,
            enabled: true,
            safe_mode: false,
        };
        
        (
            self.stage,
            SystemInstance {
                info,
                system_data: self.system_data,
                last_error: None,
            },
        )
    }
}

/// System executor state
pub struct SystemExecutor {
    stages: Shared<indexmap::IndexMap<Stage, Vec<SystemInstance>>>,
    execution_order: Shared<Vec<(Stage, Vec<usize>)>>,
}

impl SystemExecutor {
    pub fn new() -> Self {
        use indexmap::IndexMap;
        
        let mut stages = IndexMap::new();
        stages.insert(Stage::PreUpdate, Vec::new());
        stages.insert(Stage::Update, Vec::new());
        stages.insert(Stage::PostUpdate, Vec::new());
        stages.insert(Stage::PreRender, Vec::new());
        stages.insert(Stage::Render, Vec::new());
        stages.insert(Stage::PostRender, Vec::new());
        
        Self {
            stages: shared(stages),
            execution_order: shared(Vec::new()),
        }
    }
    
    pub async fn add_system(&self, stage: Stage, system: SystemInstance) {
        self.stages.write().await.entry(stage).or_insert_with(Vec::new).push(system);
        self.rebuild_execution_order().await;
    }
    
    /// Register a System (used by World::register_system)
    pub async fn register<S: System + SystemExt>(&self, system: S) -> LogicResult<()> {
        let system_id = system.system_id();
        let name = system.name().to_string();
        let dependencies = system.dependencies();
        let parallel = system.parallel();
        let system_data = SystemData::new(system);
        
        let instance = SystemInstance {
            info: SystemInfo {
                system_id,
                name,
                dependencies,
                parallel,
                retry_count: 0,
                max_retries: 3,
                enabled: true,
                safe_mode: false,
            },
            system_data,
            last_error: None,
        };
        
        self.add_system(Stage::Update, instance).await;
        Ok(())
    }
    
    async fn rebuild_execution_order(&self) {
        let stages = self.stages.read().await;
        let mut order = Vec::new();
        
        for (stage, systems) in stages.iter() {
            // Build dependency graph for this stage
            let graph = self.build_dependency_graph(systems);
            
            // Topological sort
            if let Ok(sorted) = self.topological_sort(&graph, systems.len()) {
                order.push((*stage, sorted));
            }
        }
        
        *self.execution_order.write().await = order;
    }
    
    fn build_dependency_graph(&self, systems: &[SystemInstance]) -> Vec<Vec<usize>> {
        let n = systems.len();
        let mut graph = vec![vec![]; n];
        
        for (i, system) in systems.iter().enumerate() {
            for (j, _other) in systems.iter().enumerate() {
                if i != j {
                    // Check if system depends on other
                    for _dep in &system.info.dependencies {
                        // Simplified - would check actual system IDs
                        if j < i {
                            graph[i].push(j);
                        }
                    }
                }
            }
        }
        
        graph
    }
    
    fn topological_sort(&self, graph: &[Vec<usize>], n: usize) -> LogicResult<Vec<usize>> {
        let mut in_degree = vec![0; n];
        for edges in graph {
            for &node in edges {
                in_degree[node] += 1;
            }
        }
        
        let mut queue = VecDeque::new();
        for (i, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push_back(i);
            }
        }
        
        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            result.push(node);
            
            for &neighbor in &graph[node] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }
        
        if result.len() != n {
            return Err(crate::error::LogicError::CircularDependency);
        }
        
        Ok(result)
    }
    
    pub async fn run_stage(&self, stage: Stage, world: &World, delta_time: f32) -> LogicResult<()> {
        let order = self.execution_order.read().await;
        
        for (exec_stage, system_indices) in order.iter() {
            if *exec_stage != stage {
                continue;
            }
            
            let mut stages = self.stages.write().await;
            let systems = stages.get_mut(&stage).unwrap();
            
            // Group systems by parallelism
            let mut parallel_groups = Vec::new();
            let mut current_group = Vec::new();
            
            for &idx in system_indices {
                if systems[idx].info.parallel {
                    current_group.push(idx);
                } else {
                    if !current_group.is_empty() {
                        parallel_groups.push(current_group.clone());
                        current_group.clear();
                    }
                    parallel_groups.push(vec![idx]);
                }
            }
            
            if !current_group.is_empty() {
                parallel_groups.push(current_group);
            }
            
            // Execute groups
            for group in parallel_groups {
                if group.len() == 1 {
                    // Run single system
                    let idx = group[0];
                    self.run_system(&mut systems[idx], world, delta_time).await?;
                } else {
                    // Run parallel systems
                    let mut handles = Vec::new();
                    
                    for idx in group {
                        let world_clone = world.clone();
                        let handle: JoinHandle<LogicResult<()>> = tokio::spawn(async move {
                            // In real implementation, would run the system
                            Ok(())
                        });
                        handles.push(handle);
                    }
                    
                    // Wait for all to complete
                    for handle in handles {
                        handle.await.unwrap()?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn run_system(
        &self,
        system: &mut SystemInstance,
        world: &World,
        delta_time: f32,
    ) -> LogicResult<()> {
        if !system.info.enabled {
            return Ok(());
        }
        
        // Try to run with retry logic
        for attempt in 0..=system.info.max_retries {
            match system.system_data.run(world, delta_time).await {
                Ok(()) => {
                    system.info.retry_count = 0;
                    return Ok(());
                }
                Err(e) => {
                    system.last_error = Some(e.to_string());
                    system.info.retry_count = attempt + 1;
                    
                    if attempt == system.info.max_retries {
                        // Disable system if in safe mode
                        if system.info.safe_mode {
                            system.info.enabled = false;
                        }
                        return Err(crate::error::LogicError::SystemFailure(
                            system.info.name.clone(),
                            system.info.max_retries,
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }
}