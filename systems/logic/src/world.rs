use crate::component::{ComponentRegistry, DirtyTracker, ComponentRegistration};
use crate::component_data::ComponentData;
use crate::entity::{Entity, EntityManager};
use crate::error::LogicResult;
use crate::event::EventSystem;
use crate::query::QueryBuilder;
use crate::resource_storage::ResourceStorage;
use crate::scheduler::Scheduler;
use crate::storage::HybridStorage;
use crate::system::{SystemRegistration, Stage};
use crate::system_data::SystemData;
use playground_core_types::{Handle, handle, Shared, shared};
use tokio::sync::RwLock;
use tokio::sync::RwLock as AsyncRwLock;
use std::any::TypeId;
use serde::Serialize;

/// The main ECS world that contains all entities, components, and systems
#[derive(Clone)]
pub struct World {
    entities: Handle<EntityManager>,
    storage: Handle<HybridStorage>,
    components: Handle<ComponentRegistry>,
    events: Handle<EventSystem>,
    scheduler: Handle<Scheduler>,
    dirty_tracker: Handle<DirtyTracker>,
    resources: Handle<ResourceStorage>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: handle(EntityManager::new()),
            storage: handle(HybridStorage::new()),
            components: handle(ComponentRegistry::new()),
            events: handle(EventSystem::new()),
            scheduler: handle(Scheduler::new()),
            dirty_tracker: handle(DirtyTracker::new()),
            resources: handle(ResourceStorage::new()),
        }
    }
    
    /// Register a component type
    pub fn register_component<T: crate::component::Component>(&self) -> ComponentRegistration {
        ComponentRegistration::new::<T>()
    }
    
    /// Spawn entities in batch
    pub async fn spawn_batch(&self, count: usize) -> Vec<Entity> {
        let entities = self.entities.create_batch(count).await;
        
        // Send spawn events
        for entity in &entities {
            self.events.send(
                *entity,
                crate::event::EntitySpawned { entity: *entity },
            ).await;
        }
        
        entities
    }
    
    /// Spawn entity with components
    pub async fn spawn_with<F>(&self, builder: F) -> LogicResult<Entity>
    where
        F: FnOnce() -> Vec<ComponentData>,
    {
        let entity = self.entities.create().await;
        let components = builder();
        
        self.storage.spawn_entity(entity, components).await?;
        
        self.events.send(
            entity,
            crate::event::EntitySpawned { entity },
        ).await;
        
        Ok(entity)
    }
    
    /// Despawn entities in batch
    pub async fn despawn_batch(&self, entities: &[Entity]) -> LogicResult<()> {
        for entity in entities {
            self.storage.despawn_entity(*entity).await?;
            self.entities.destroy(*entity).await?;
            
            self.events.send(
                *entity,
                crate::event::EntityDespawned { entity: *entity },
            ).await;
        }
        
        Ok(())
    }
    
    /// Add component to entity
    pub async fn add_component<T: 'static + Send + Sync>(
        &self,
        entity: Entity,
        component: T,
    ) -> LogicResult<()> {
        let type_id = TypeId::of::<T>();
        
        self.storage.add_component(entity, component).await?;
        
        // Track if networked
        if self.components.is_networked(type_id).await {
            self.dirty_tracker.mark_dirty(entity, type_id).await;
        }
        
        self.events.send(
            entity,
            crate::event::ComponentAdded {
                entity,
                component_type: type_id,
            },
        ).await;
        
        Ok(())
    }
    
    /// Remove component from entity
    pub async fn remove_component<T: 'static>(&self, entity: Entity) -> LogicResult<()> {
        let type_id = TypeId::of::<T>();
        
        self.storage.remove_component::<T>(entity).await?;
        
        self.events.send(
            entity,
            crate::event::ComponentRemoved {
                entity,
                component_type: type_id,
            },
        ).await;
        
        Ok(())
    }
    
    /// Check if entity has component
    pub async fn has_component<T: 'static>(&self, entity: Entity) -> bool {
        self.storage.has_component::<T>(entity).await
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
    pub async fn insert_resource<R: 'static + Send + Sync + Serialize>(&self, resource: R) -> LogicResult<()> {
        self.resources.insert(resource).await
    }
    
    /// Check if a global resource exists
    pub async fn has_resource<R: 'static>(&self) -> bool {
        self.resources.contains::<R>().await
    }
    
    /// Run one frame of the world
    pub async fn update(&self, delta_time: f32) -> LogicResult<()> {
        // Process events from last frame
        self.events.process_events().await;
        
        // Run systems
        self.scheduler.run_frame(self, delta_time).await?;
        
        // Clear frame events
        self.events.clear_frame_events().await;
        
        // Incremental GC
        self.run_incremental_gc().await?;
        
        Ok(())
    }
    
    /// Get dirty entities for networking
    pub async fn get_dirty_entities(&self, max_count: usize) -> Vec<(Entity, Vec<TypeId>)> {
        self.dirty_tracker.get_dirty_batch(max_count).await
    }
    
    /// Clear all entities and components
    pub async fn clear(&self) {
        self.entities.clear().await;
        // Storage clear would go here
        self.dirty_tracker.clear().await;
    }
    
    /// Get entity count
    pub async fn entity_count(&self) -> usize {
        self.entities.count().await
    }
    
    /// Check if entity is alive
    pub async fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity).await
    }
    
    /// Run incremental garbage collection
    async fn run_incremental_gc(&self) -> LogicResult<()> {
        // This would implement the incremental GC logic
        // For now, just a placeholder
        Ok(())
    }
    
    /// Register a System with the World (for plugins)
    pub async fn register_plugin_system(&mut self, system: SystemData) -> LogicResult<()> {
        // Register with the scheduler
        self.scheduler.executor().register(system).await
    }
    
    /// Run all registered systems for one frame
    pub async fn run_systems(&mut self, delta_time: f32) -> LogicResult<()> {
        self.scheduler.run_frame(self, delta_time).await
    }
}


/// The main ECS facade for plugins and apps
pub struct ECS {
    world: Handle<World>,
    systems: Option<Handle<crate::systems_manager::SystemsManager>>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            world: handle(World::new()),
            systems: None,
        }
    }
    
    /// Initialize all engine systems
    /// This must be called before using any system functionality
    pub async fn initialize_systems(&mut self) -> crate::error::LogicResult<Handle<crate::systems_manager::SystemsManager>> {
        let world_lock = shared(self.world.as_ref().clone());
        let systems = crate::systems_manager::SystemsManager::new(world_lock).await?;
        let systems = handle(systems);
        self.systems = Some(systems.clone());
        Ok(systems)
    }
    
    /// Get the systems manager
    pub fn systems(&self) -> Option<&Handle<crate::systems_manager::SystemsManager>> {
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
    world: Handle<World>,
    components: Vec<ComponentData>,
}

impl EntitySpawner {
    fn new(world: Handle<World>) -> Self {
        Self {
            world,
            components: Vec::new(),
        }
    }
    
    pub fn with<T: crate::component::Component + Serialize + 'static + Send + Sync>(mut self, component: T) -> LogicResult<Self> {
        let data = ComponentData::new(component)?;
        self.components.push(data);
        Ok(self)
    }
    
    pub async fn spawn(self) -> LogicResult<Entity> {
        self.world.spawn_with(|| self.components).await
    }
    
    pub async fn spawn_batch(self, count: usize) -> LogicResult<Vec<Entity>> {
        // For batch spawning, we can only use the base entity without components
        // since components can't be cloned
        Ok(self.world.spawn_batch(count).await)
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
            $(ComponentData::new($component).unwrap()),+
        ]
    }};
}