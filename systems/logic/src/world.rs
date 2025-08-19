use crate::component::{ComponentRegistry, DirtyTracker, ComponentRegistration};
use crate::entity::{Entity, EntityManager};
use crate::error::LogicResult;
use crate::event::EventSystem;
use crate::query::QueryBuilder;
use crate::scheduler::Scheduler;
use crate::storage::HybridStorage;
use crate::system::{SystemRegistration, Stage};
use parking_lot::RwLock;
use std::any::TypeId;
use std::sync::Arc;

/// The main ECS world that contains all entities, components, and systems
#[derive(Clone)]
pub struct World {
    entities: Arc<EntityManager>,
    storage: Arc<HybridStorage>,
    components: Arc<ComponentRegistry>,
    events: Arc<EventSystem>,
    scheduler: Arc<Scheduler>,
    dirty_tracker: Arc<DirtyTracker>,
    resources: Arc<RwLock<fnv::FnvHashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>>,
}

use fnv::FnvHashMap;

impl World {
    pub fn new() -> Self {
        Self {
            entities: Arc::new(EntityManager::new()),
            storage: Arc::new(HybridStorage::new()),
            components: Arc::new(ComponentRegistry::new()),
            events: Arc::new(EventSystem::new()),
            scheduler: Arc::new(Scheduler::new()),
            dirty_tracker: Arc::new(DirtyTracker::new()),
            resources: Arc::new(RwLock::new(FnvHashMap::default())),
        }
    }
    
    /// Register a component type
    pub fn register_component<T: crate::component::Component>(&self) -> ComponentRegistration {
        ComponentRegistration::new::<T>()
    }
    
    /// Spawn entities in batch
    pub fn spawn_batch(&self, count: usize) -> Vec<Entity> {
        let entities = self.entities.create_batch(count);
        
        // Send spawn events
        for entity in &entities {
            self.events.send(
                *entity,
                crate::event::EntitySpawned { entity: *entity },
            );
        }
        
        entities
    }
    
    /// Spawn entity with components
    pub fn spawn_with<F>(&self, builder: F) -> LogicResult<Entity>
    where
        F: FnOnce() -> Vec<(TypeId, Box<dyn std::any::Any + Send + Sync>)>,
    {
        let entity = self.entities.create();
        let components = builder();
        
        self.storage.spawn_entity(entity, components)?;
        
        self.events.send(
            entity,
            crate::event::EntitySpawned { entity },
        );
        
        Ok(entity)
    }
    
    /// Despawn entities in batch
    pub fn despawn_batch(&self, entities: &[Entity]) -> LogicResult<()> {
        for entity in entities {
            self.storage.despawn_entity(*entity)?;
            self.entities.destroy(*entity)?;
            
            self.events.send(
                *entity,
                crate::event::EntityDespawned { entity: *entity },
            );
        }
        
        Ok(())
    }
    
    /// Add component to entity
    pub fn add_component<T: 'static + Send + Sync>(
        &self,
        entity: Entity,
        component: T,
    ) -> LogicResult<()> {
        let type_id = TypeId::of::<T>();
        
        self.storage.add_component(entity, component)?;
        
        // Track if networked
        if self.components.is_networked(type_id) {
            self.dirty_tracker.mark_dirty(entity, type_id);
        }
        
        self.events.send(
            entity,
            crate::event::ComponentAdded {
                entity,
                component_type: type_id,
            },
        );
        
        Ok(())
    }
    
    /// Remove component from entity
    pub fn remove_component<T: 'static>(&self, entity: Entity) -> LogicResult<()> {
        let type_id = TypeId::of::<T>();
        
        self.storage.remove_component::<T>(entity)?;
        
        self.events.send(
            entity,
            crate::event::ComponentRemoved {
                entity,
                component_type: type_id,
            },
        );
        
        Ok(())
    }
    
    /// Check if entity has component
    pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
        self.storage.has_component::<T>(entity)
    }
    
    /// Create a query builder
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new(&self.storage)
    }
    
    /// Add a system to the world
    pub fn add_system<S: crate::system::System>(&self, system: S) -> SystemRegistration {
        SystemRegistration::new(system)
    }
    
    /// Register a system with stage
    pub fn register_system(&self, stage: Stage, registration: SystemRegistration) {
        let (stage, instance) = registration.build();
        self.scheduler.executor().add_system(stage, instance);
    }
    
    /// Insert a global resource
    pub fn insert_resource<R: 'static + Send + Sync>(&self, resource: R) {
        self.resources
            .write()
            .insert(TypeId::of::<R>(), Box::new(resource));
    }
    
    /// Check if a global resource exists
    pub fn has_resource<R: 'static>(&self) -> bool {
        self.resources.read().contains_key(&TypeId::of::<R>())
    }
    
    /// Run one frame of the world
    pub async fn update(&self, delta_time: f32) -> LogicResult<()> {
        // Process events from last frame
        self.events.process_events();
        
        // Run systems
        self.scheduler.run_frame(self, delta_time).await?;
        
        // Clear frame events
        self.events.clear_frame_events();
        
        // Incremental GC
        self.run_incremental_gc().await?;
        
        Ok(())
    }
    
    /// Get dirty entities for networking
    pub fn get_dirty_entities(&self, max_count: usize) -> Vec<(Entity, Vec<TypeId>)> {
        self.dirty_tracker.get_dirty_batch(max_count)
    }
    
    /// Clear all entities and components
    pub fn clear(&self) {
        self.entities.clear();
        // Storage clear would go here
        self.dirty_tracker.clear();
    }
    
    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.entities.count()
    }
    
    /// Check if entity is alive
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }
    
    /// Run incremental garbage collection
    async fn run_incremental_gc(&self) -> LogicResult<()> {
        // This would implement the incremental GC logic
        // For now, just a placeholder
        Ok(())
    }
}


/// The main ECS facade for plugins and apps
pub struct ECS {
    world: Arc<World>,
    systems: Option<Arc<crate::systems_manager::SystemsManager>>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            world: Arc::new(World::new()),
            systems: None,
        }
    }
    
    /// Initialize all engine systems
    /// This must be called before using any system functionality
    pub async fn initialize_systems(&mut self) -> crate::error::LogicResult<Arc<crate::systems_manager::SystemsManager>> {
        let systems = crate::systems_manager::SystemsManager::initialize().await?;
        let systems = Arc::new(systems);
        self.systems = Some(systems.clone());
        Ok(systems)
    }
    
    /// Get the systems manager
    pub fn systems(&self) -> Option<&Arc<crate::systems_manager::SystemsManager>> {
        self.systems.as_ref()
    }
    
    pub fn world(&self) -> &World {
        &self.world
    }
    
    /// Spawn entities using builder pattern
    pub fn spawn(&self) -> EntitySpawner {
        EntitySpawner::new(self.world.clone())
    }
    
    /// Create a query
    pub fn query(&self) -> QueryBuilder {
        self.world.query()
    }
    
    /// Run the ECS for one frame
    pub async fn update(&self, delta_time: f32) -> LogicResult<()> {
        self.world.update(delta_time).await
    }
}

/// Builder for spawning entities
pub struct EntitySpawner {
    world: Arc<World>,
    components: Vec<(TypeId, Box<dyn std::any::Any + Send + Sync>)>,
}

impl EntitySpawner {
    fn new(world: Arc<World>) -> Self {
        Self {
            world,
            components: Vec::new(),
        }
    }
    
    pub fn with<T: 'static + Send + Sync>(mut self, component: T) -> Self {
        self.components.push((TypeId::of::<T>(), Box::new(component)));
        self
    }
    
    pub fn spawn(self) -> LogicResult<Entity> {
        self.world.spawn_with(|| self.components)
    }
    
    pub fn spawn_batch(self, count: usize) -> LogicResult<Vec<Entity>> {
        // For batch spawning, we can only use the base entity without components
        // since components can't be cloned
        Ok(self.world.spawn_batch(count))
    }
}

/// Macro for spawning entities with components
#[macro_export]
macro_rules! spawn {
    ($world:expr, $($component:expr),+) => {{
        $world.spawn()
            $(.with($component))+
            .spawn()
    }};
}

/// Macro for creating component bundles
#[macro_export]
macro_rules! bundle {
    ($($component:expr),+) => {{
        vec![
            $((std::any::TypeId::of_val(&$component), Box::new($component) as Box<dyn std::any::Any + Send + Sync>)),+
        ]
    }};
}