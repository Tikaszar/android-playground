use crate::component::{ComponentRegistry, DirtyTracker, ComponentRegistration};
use crate::component::{Component, ComponentData};
use crate::entity::{Entity, EntityManager};
use crate::error::LogicResult;
use crate::event::EventSystem;
use crate::query::QueryBuilder;
use crate::resource_storage::ResourceStorage;
use crate::scheduler::Scheduler;
use crate::storage::HybridStorage;
use crate::system::{SystemRegistration, Stage};
use crate::system_data::SystemData;
use playground_core_types::{Handle, handle, shared};
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
    // Plugin channel IDs for lifecycle management (no dyn!)
    plugin_channels: playground_core_types::Shared<Vec<(String, u16)>>, // (name, channel_id)
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
            plugin_channels: shared(Vec::new()),
        }
    }
    
    /// Register a component type
    pub fn register_component<T: crate::component::ComponentData>(&self) -> ComponentRegistration {
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
        F: FnOnce() -> Vec<Component>,
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
    pub async fn add_component<T: ComponentData + serde::Serialize + 'static + Send + Sync>(
        &self,
        entity: Entity,
        component: T,
    ) -> LogicResult<()> {
        let component_name = T::component_name();
        
        self.storage.add_component(entity, component).await?;
        
        // Note: ComponentRegistry and DirtyTracker still use TypeId internally
        // This is a known architectural mismatch that needs to be fixed
        // For now, we skip networked tracking since we can't convert ComponentId to TypeId
        
        self.events.send(
            entity,
            crate::event::ComponentAdded {
                entity,
                component_type_name: component_name.to_string(),
            },
        ).await;
        
        Ok(())
    }
    
    /// Remove component from entity
    pub async fn remove_component<T: ComponentData + 'static>(&self, entity: Entity) -> LogicResult<()> {
        let component_id = T::component_id();
        let component_name = T::component_name();
        
        self.storage.remove_component_by_id(entity, component_id).await?;
        
        self.events.send(
            entity,
            crate::event::ComponentRemoved {
                entity,
                component_type_name: component_name.to_string(),
            },
        ).await;
        
        Ok(())
    }
    
    /// Check if entity has component
    pub async fn has_component<T: ComponentData + 'static>(&self, entity: Entity) -> bool {
        let component_id = T::component_id();
        self.storage.has_component_by_id(entity, component_id).await
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
    pub fn register_system(&self, _stage: Stage, registration: SystemRegistration) {
        let (stage, instance) = registration.build();
        self.scheduler.executor().add_system(stage, instance);
    }
    
    /// Insert a global resource
    pub async fn insert_resource<R: crate::resource_storage::Resource + 'static + Send + Sync + Serialize>(&self, resource: R) -> LogicResult<()> {
        self.resources.insert(resource).await
    }
    
    /// Check if a global resource exists
    pub async fn has_resource<R: crate::resource_storage::Resource + 'static>(&self) -> bool {
        let resource_id = R::resource_id();
        self.resources.contains(&resource_id).await
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
    pub async fn get_dirty_entities(&self, max_count: usize) -> Vec<(Entity, Vec<String>)> {
        // DirtyTracker returns TypeId which we need to convert to String
        let dirty_batch = self.dirty_tracker.get_dirty_batch(max_count).await;
        dirty_batch.into_iter()
            .map(|(entity, type_ids)| {
                let type_strings = type_ids.into_iter()
                    .map(|type_id| format!("{:?}", type_id))
                    .collect();
                (entity, type_strings)
            })
            .collect()
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
    
    /// Register a plugin channel for lifecycle tracking
    /// Plugins themselves run as independent tasks
    pub async fn register_plugin_channel(&mut self, name: String, channel_id: u16) -> LogicResult<()> {
        let mut channels = self.plugin_channels.write().await;
        channels.push((name, channel_id));
        Ok(())
    }
    
    /// Get all registered plugin channels
    pub async fn get_plugin_channels(&self) -> Vec<(String, u16)> {
        let channels = self.plugin_channels.read().await;
        channels.clone()
    }
    
    /// Run all registered systems for one frame
    pub async fn run_systems(&mut self, delta_time: f32) -> LogicResult<()> {
        // Run scheduler systems
        self.scheduler.run_frame(self, delta_time).await?;
        
        // Plugins run independently in their own tasks, not called from here
        // They communicate via channels
        
        Ok(())
    }
    
    /// Cleanup all systems on shutdown
    pub async fn shutdown(&mut self) -> LogicResult<()> {
        // Send shutdown messages to plugin channels
        // This would be implemented via the networking system
        let mut channels = self.plugin_channels.write().await;
        channels.clear();
        
        // Clear other resources
        self.clear().await;
        
        Ok(())
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
    components: Vec<Component>,
}

impl EntitySpawner {
    fn new(world: Handle<World>) -> Self {
        Self {
            world,
            components: Vec::new(),
        }
    }
    
    pub async fn with<T: crate::component::ComponentData + Serialize + 'static + Send + Sync>(mut self, component: T) -> LogicResult<Self> {
        let data = Component::new(component).await?;
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

// Note: bundle macro removed - use async functions to create component bundles instead
// Components must be created with Component::new() which is async