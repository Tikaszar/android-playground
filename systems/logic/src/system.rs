use crate::error::LogicResult;
use crate::world::World;
use parking_lot::RwLock;
use std::any::TypeId;
use std::sync::Arc;
use tokio::task::JoinHandle;

/// System trait that all game systems must implement
#[async_trait::async_trait]
pub trait System: Send + Sync + 'static {
    /// System name for debugging
    fn name(&self) -> &'static str;
    
    /// System dependencies - systems that must run before this one
    fn dependencies(&self) -> Vec<TypeId> {
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

/// System metadata for runtime management
pub struct SystemInfo {
    pub type_id: TypeId,
    pub name: String,
    pub dependencies: Vec<TypeId>,
    pub parallel: bool,
    pub retry_count: u32,
    pub max_retries: u32,
    pub enabled: bool,
    pub safe_mode: bool,
}

/// System instance wrapper
pub struct SystemInstance {
    pub info: SystemInfo,
    pub system: Box<dyn System>,
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
    system: Box<dyn System>,
    stage: Stage,
    dependencies: Vec<TypeId>,
    before: Vec<TypeId>,
    after: Vec<TypeId>,
}

impl SystemRegistration {
    pub fn new<S: System>(system: S) -> Self {
        let dependencies = system.dependencies();
        Self {
            system: Box::new(system),
            stage: Stage::Update,
            dependencies,
            before: Vec::new(),
            after: Vec::new(),
        }
    }
    
    pub fn stage(mut self, stage: Stage) -> Self {
        self.stage = stage;
        self
    }
    
    pub fn depends_on<S: System + 'static>(mut self) -> Self {
        self.dependencies.push(TypeId::of::<S>());
        self
    }
    
    pub fn runs_before<S: System + 'static>(mut self) -> Self {
        self.before.push(TypeId::of::<S>());
        self
    }
    
    pub fn runs_after<S: System + 'static>(mut self) -> Self {
        self.after.push(TypeId::of::<S>());
        self
    }
    
    pub fn build(self) -> (Stage, SystemInstance) {
        let info = SystemInfo {
            type_id: TypeId::of::<Box<dyn System>>(), // Simplified
            name: self.system.name().to_string(),
            dependencies: self.dependencies,
            parallel: self.system.parallel(),
            retry_count: 0,
            max_retries: 3,
            enabled: true,
            safe_mode: false,
        };
        
        (
            self.stage,
            SystemInstance {
                info,
                system: self.system,
                last_error: None,
            },
        )
    }
}

/// System executor state
pub struct SystemExecutor {
    stages: Arc<RwLock<indexmap::IndexMap<Stage, Vec<SystemInstance>>>>,
    execution_order: Arc<RwLock<Vec<(Stage, Vec<usize>)>>>,
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
            stages: Arc::new(RwLock::new(stages)),
            execution_order: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn add_system(&self, stage: Stage, system: SystemInstance) {
        self.stages.write().entry(stage).or_insert_with(Vec::new).push(system);
        self.rebuild_execution_order();
    }
    
    fn rebuild_execution_order(&self) {
        let stages = self.stages.read();
        let mut order = Vec::new();
        
        for (stage, systems) in stages.iter() {
            // Build dependency graph for this stage
            let graph = self.build_dependency_graph(systems);
            
            // Topological sort
            if let Ok(sorted) = self.topological_sort(&graph, systems.len()) {
                order.push((*stage, sorted));
            }
        }
        
        *self.execution_order.write() = order;
    }
    
    fn build_dependency_graph(&self, systems: &[SystemInstance]) -> Vec<Vec<usize>> {
        let n = systems.len();
        let mut graph = vec![vec![]; n];
        
        for (i, system) in systems.iter().enumerate() {
            for (j, other) in systems.iter().enumerate() {
                if i != j {
                    // Check if system depends on other
                    for dep in &system.info.dependencies {
                        // Simplified - would check actual type IDs
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
        let order = self.execution_order.read();
        
        for (exec_stage, system_indices) in order.iter() {
            if *exec_stage != stage {
                continue;
            }
            
            let mut stages = self.stages.write();
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
            match system.system.run(world, delta_time).await {
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

use std::collections::VecDeque;