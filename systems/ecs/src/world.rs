//! Unified World implementation for the entire engine

use std::collections::{HashMap, VecDeque};
use playground_core_types::{Handle, handle, Shared, shared};
use async_trait::async_trait;
use playground_core_ecs::{
    ComponentData, ComponentId, EntityId, 
    EcsError, EcsResult, WorldContract, StorageType,
    WorldCommand, WorldCommandHandler
};
use crate::storage::ComponentStorage;
use crate::entity::EntityAllocator;
use crate::component::{ComponentBox, ComponentRegistry};
use crate::query::QueryBuilder;
use crate::scheduler::SystemScheduler;
use crate::messaging::MessageBus;
use playground_core_ecs::ExecutionStage;
use bytes::Bytes;

/// Memory statistics for the World
pub struct MemoryStats {
    pub total_entities: usize,
    pub total_components: usize,
    pub pool_usage: usize,
    pub pool_limit: usize,
    pub growth_rate: f32,
}

/// Incremental garbage collector for entity cleanup
pub struct GarbageCollector {
    dead_entities: Shared<VecDeque<EntityId>>,
    frame_budget_ms: u64,
    enabled: bool,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            dead_entities: shared(VecDeque::new()),
            frame_budget_ms: 2,
            enabled: true,
        }
    }
    
    pub async fn queue_for_collection(&self, entity: EntityId) {
        self.dead_entities.write().await.push_back(entity);
    }
    
    pub async fn collect_incremental(&self, world: &World) -> EcsResult<usize> {
        if !self.enabled {
            return Ok(0);
        }
        
        let start = std::time::Instant::now();
        let mut collected = 0;
        
        while start.elapsed().as_millis() < self.frame_budget_ms as u128 {
            let entity = {
                let mut dead = self.dead_entities.write().await;
                dead.pop_front()
            };
            
            if let Some(entity) = entity {
                world.despawn_immediate(entity).await?;
                collected += 1;
            } else {
                break;
            }
        }
        
        Ok(collected)
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn set_frame_budget(&mut self, ms: u64) {
        self.frame_budget_ms = ms;
    }
}

/// The unified World implementation for the entire engine
/// 
/// This is the single source of truth for all entities and components
/// in the Android Playground engine. All systems and plugins interact
/// with this World through the systems/logic API gateway.
pub struct World {
    // Entity management
    entities: Shared<HashMap<EntityId, Vec<ComponentId>>>,
    allocator: Shared<EntityAllocator>,
    
    // Component storage
    storages: Shared<HashMap<ComponentId, Handle<ComponentStorage>>>,
    registry: Handle<ComponentRegistry>,
    
    // System scheduling
    scheduler: Handle<SystemScheduler>,
    
    // Messaging system (core ECS functionality)
    message_bus: Handle<MessageBus>,
    
    // Memory management
    gc: Handle<GarbageCollector>,
    memory_stats: Shared<MemoryStats>,
}

impl World {
    /// Create a new World instance
    pub fn new() -> Self {
        Self::with_registry(handle(ComponentRegistry::new()))
    }
    
    /// Create a new World with a custom component registry
    pub fn with_registry(registry: Handle<ComponentRegistry>) -> Self {
        Self {
            entities: shared(HashMap::new()),
            storages: shared(HashMap::new()),
            allocator: shared(EntityAllocator::new()),
            registry,
            scheduler: handle(SystemScheduler::new()),
            message_bus: handle(MessageBus::new()),
            gc: handle(GarbageCollector::new()),
            memory_stats: shared(MemoryStats {
                total_entities: 0,
                total_components: 0,
                pool_usage: 0,
                pool_limit: 100 * 1024 * 1024, // 100MB default
                growth_rate: 0.0,
            }),
        }
    }
    
    /// Register a component type with the World
    pub async fn register_component<T: ComponentData>(&self) -> EcsResult<()> {
        self.registry.register::<T>().await?;
        
        let component_id = T::component_id();
        if !self.storages.read().await.contains_key(&component_id) {
            let storage = handle(ComponentStorage::new_sparse(component_id.clone()));
            self.storages.write().await.insert(component_id, storage);
        }
        
        Ok(())
    }
    
    /// Spawn a batch of entities with components
    pub async fn spawn_batch(&self, bundles: Vec<Vec<ComponentBox>>) -> EcsResult<Vec<EntityId>> {
        let count = bundles.len();
        let entities = {
            let allocator = self.allocator.read().await;
            allocator.allocate_batch(count).await
        };
        
        for (entity, bundle) in entities.iter().zip(bundles) {
            let mut component_ids = Vec::new();
            
            for component in bundle {
                let component_id = component.component_id();
                let type_name = component.component_name();
                
                component_ids.push(component_id.clone());
                
                // Clone storage reference to avoid holding lock across await
                let storage = {
                    let storages = self.storages.read().await;
                    storages.get(&component_id).cloned()
                };
                
                if let Some(storage) = storage {
                    storage.insert(*entity, component).await?;
                } else {
                    return Err(EcsError::ComponentNotRegistered(type_name.to_string()));
                }
            }
            
            self.entities.write().await.insert(*entity, component_ids);
        }
        
        let mut stats = self.memory_stats.write().await;
        stats.total_entities += count;
        
        Ok(entities)
    }
    
    /// Queue entities for despawning
    pub async fn despawn_batch(&self, entities: Vec<EntityId>) -> EcsResult<()> {
        for entity in entities {
            self.gc.queue_for_collection(entity).await;
        }
        Ok(())
    }
    
    /// Immediately despawn an entity
    async fn despawn_immediate(&self, entity: EntityId) -> EcsResult<()> {
        if let Some(component_ids) = self.entities.write().await.remove(&entity) {
            for component_id in component_ids {
                // Clone storage reference to avoid holding lock across await
                let storage = {
                    let storages = self.storages.read().await;
                    storages.get(&component_id).cloned()
                };
                
                if let Some(storage) = storage {
                    let _ = storage.remove(entity).await;
                }
            }
            
            {
                let allocator = self.allocator.read().await;
                allocator.free(entity).await;
            }
            
            let mut stats = self.memory_stats.write().await;
            stats.total_entities = stats.total_entities.saturating_sub(1);
        }
        
        Ok(())
    }
    
    /// Add a component to an entity
    pub async fn add_component_raw(&self, entity: EntityId, component: ComponentBox, component_id: ComponentId) -> EcsResult<()> {
        // Clone the storage reference to avoid holding lock across await
        let storage = {
            let storages = self.storages.read().await;
            storages.get(&component_id).cloned()
        };
        
        if let Some(storage) = storage {
            storage.insert(entity, component).await?;
            
            // Update entity's component list
            let mut entities = self.entities.write().await;
            if let Some(components) = entities.get_mut(&entity) {
                if !components.contains(&component_id) {
                    components.push(component_id.clone());
                }
            }
            
            Ok(())
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    /// Remove a component from an entity
    pub async fn remove_component_raw(&self, entity: EntityId, component_id: ComponentId) -> EcsResult<ComponentBox> {
        // Clone the storage reference to avoid holding lock across await
        let storage = {
            let storages = self.storages.read().await;
            storages.get(&component_id).cloned()
        };
        
        if let Some(storage) = storage {
            let component_box = storage.remove(entity).await?;
            
            // Update entity's component list
            let mut entities = self.entities.write().await;
            if let Some(components) = entities.get_mut(&entity) {
                components.retain(|id| *id != component_id);
            }
            
            Ok(component_box)
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    /// Get a component from an entity
    pub async fn get_component_raw(&self, entity: EntityId, component_id: ComponentId) -> EcsResult<ComponentBox> {
        // Clone the storage reference to avoid holding lock across await
        let storage = {
            let storages = self.storages.read().await;
            storages.get(&component_id).cloned()
        };
        
        if let Some(storage) = storage {
            storage.get_raw(entity).await
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    /// Get a typed component from an entity
    pub async fn get_component<T: ComponentData>(&self, entity: EntityId) -> EcsResult<T> {
        let component_box = self.get_component_raw(entity, T::component_id()).await?;
        // Deserialize from bytes
        let bytes = component_box.serialize();
        T::deserialize(&bytes).await.map_err(|e| e.into())
    }
    
    /// Check if an entity has a component
    pub async fn has_component(&self, entity: EntityId, component_id: ComponentId) -> bool {
        // Clone the storage reference to avoid holding lock across await
        let storage = {
            let storages = self.storages.read().await;
            storages.get(&component_id).cloned()
        };
        
        if let Some(storage) = storage {
            storage.contains(entity).await
        } else {
            false
        }
    }
    
    /// Create a new query builder
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new()
    }
    
    /// Execute a query to find entities
    pub async fn execute_query(&self, query: &QueryBuilder) -> EcsResult<Vec<EntityId>> {
        query.execute(&self.storages).await
    }
    
    /// Run garbage collection
    pub async fn run_gc(&self) -> EcsResult<usize> {
        self.gc.collect_incremental(self).await
    }
    
    /// Get entities with dirty components
    pub async fn get_dirty_entities(&self) -> Vec<(EntityId, Vec<ComponentId>)> {
        let mut result: Vec<(EntityId, Vec<ComponentId>)> = Vec::new();
        
        // Clone storage references to avoid holding lock across await
        let storages_list: Vec<(ComponentId, Handle<ComponentStorage>)> = {
            let storages = self.storages.read().await;
            storages.iter().map(|(id, storage)| (id.clone(), storage.clone())).collect()
        };
        
        for (component_id, storage) in storages_list {
            for entity in storage.get_dirty().await {
                let exists = result.iter_mut().find(|(e, _)| *e == entity);
                if let Some((_, components)) = exists {
                    components.push(component_id.clone());
                } else {
                    result.push((entity, vec![component_id.clone()]));
                }
            }
        }
        
        result
    }
    
    /// Clear dirty flags on all components
    pub async fn clear_dirty(&self) -> EcsResult<()> {
        // Clone storage references to avoid holding lock across await
        let storages_list: Vec<Handle<ComponentStorage>> = {
            let storages = self.storages.read().await;
            storages.values().cloned().collect()
        };
        
        for storage in storages_list {
            storage.clear_dirty().await?;
        }
        Ok(())
    }
    
    /// Get memory statistics
    pub async fn memory_stats(&self) -> MemoryStats {
        let (total_entities, growth_rate) = {
            let stats = self.memory_stats.read().await;
            (stats.total_entities, stats.growth_rate)
        };
        
        let total_components = self.storages.read().await.len();
        
        let pool_usage = self.registry.current_pool_usage().await;
        let pool_limit = self.registry.pool_limit();
        
        MemoryStats {
            total_entities,
            total_components,
            pool_usage,
            pool_limit,
            growth_rate,
        }
    }
    
    /// Check if the World is empty
    pub async fn is_empty(&self) -> bool {
        self.entities.read().await.is_empty()
    }
    
    /// Get the system scheduler
    pub fn scheduler(&self) -> Handle<SystemScheduler> {
        self.scheduler.clone()
    }
    
    /// Execute systems for a specific stage
    pub async fn execute_stage(&self, stage: ExecutionStage, delta_time: f32) -> EcsResult<()> {
        self.scheduler.execute_stage(stage, self, delta_time).await
    }
    
    /// Execute all stages in order
    pub async fn update(&self, delta_time: f32) -> EcsResult<()> {
        for stage in ExecutionStage::all() {
            self.execute_stage(*stage, delta_time).await?;
        }
        Ok(())
    }
    
    // Messaging API - Core ECS functionality
    
    /// Get the message bus
    pub fn message_bus(&self) -> Handle<MessageBus> {
        self.message_bus.clone()
    }
    
    /// Publish a message to a channel
    pub async fn publish(&self, channel: playground_core_ecs::ChannelId, message: impl Into<Bytes>) -> EcsResult<()> {
        use playground_core_ecs::MessageBusContract;
        self.message_bus.publish(channel, message.into()).await
    }
    
    /// Subscribe to a channel
    pub async fn subscribe(&self, channel: playground_core_ecs::ChannelId, handler_id: String) -> EcsResult<()> {
        use playground_core_ecs::MessageBusContract;
        self.message_bus.subscribe(channel, handler_id).await
    }
    
    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: playground_core_ecs::ChannelId, handler_id: &str) -> EcsResult<()> {
        use playground_core_ecs::MessageBusContract;
        self.message_bus.unsubscribe(channel, handler_id).await
    }
    
    // Command processor functionality
    
    /// Start the command processor for this World
    /// This allows systems to interact with the World through core/ecs
    pub fn start_command_processor(self: std::sync::Arc<Self>) {
        use tokio::sync::mpsc;
        
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Register the sender with core/ecs
        playground_core_ecs::register_command_sender(tx);
        
        // Spawn the command processor task
        let world = self;
        tokio::spawn(async move {
            while let Some(command) = rx.recv().await {
                world.handle_command(command).await;
            }
        });
    }
    
    /// Spawn a single entity without components
    pub async fn spawn_entity(&self) -> EcsResult<EntityId> {
        let entity = {
            let allocator = self.allocator.read().await;
            allocator.allocate().await
        };
        
        self.entities.write().await.insert(entity, Vec::new());
        
        let mut stats = self.memory_stats.write().await;
        stats.total_entities += 1;
        
        Ok(entity)
    }
    
    /// Get a component as bytes (for command processor)
    pub async fn get_component_bytes(&self, entity: EntityId, component_id: ComponentId) -> EcsResult<Bytes> {
        // Check if entity exists
        if !self.entities.read().await.contains_key(&entity) {
            return Err(EcsError::EntityNotFound(entity));
        }
        
        // Get the storage for this component type
        let storage = {
            let storages = self.storages.read().await;
            storages.get(&component_id).cloned()
        };
        
        if let Some(storage) = storage {
            storage.get_bytes(entity).await
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    /// Set a component from bytes (for command processor)
    pub async fn set_component_bytes(&self, entity: EntityId, component_id: ComponentId, data: Bytes) -> EcsResult<()> {
        // Check if entity exists
        if !self.entities.read().await.contains_key(&entity) {
            return Err(EcsError::EntityNotFound(entity));
        }
        
        // Get the storage for this component type
        let storage = {
            let storages = self.storages.read().await;
            storages.get(&component_id).cloned()
        };
        
        if let Some(storage) = storage {
            // Create a ComponentBox from the bytes
            let component = crate::component::component_box::from_bytes(component_id.clone(), data);
            storage.insert(entity, component).await?;
            
            // Update entity's component list
            let mut entities = self.entities.write().await;
            if let Some(components) = entities.get_mut(&entity) {
                if !components.contains(&component_id) {
                    components.push(component_id);
                }
            }
            
            Ok(())
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    /// Add a component from bytes
    pub async fn add_component_bytes(&self, entity: EntityId, component_id: ComponentId, data: Bytes) -> EcsResult<()> {
        self.set_component_bytes(entity, component_id, data).await
    }
    
    /// Remove a component from an entity
    pub async fn remove_component(&self, entity: EntityId, component_id: ComponentId) -> EcsResult<()> {
        // Get the storage for this component type
        let storage = {
            let storages = self.storages.read().await;
            storages.get(&component_id).cloned()
        };
        
        if let Some(storage) = storage {
            storage.remove(entity).await?;
            
            // Update entity's component list
            let mut entities = self.entities.write().await;
            if let Some(components) = entities.get_mut(&entity) {
                components.retain(|c| c != &component_id);
            }
            
            Ok(())
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    /// Check if an entity is alive
    pub async fn is_alive(&self, entity: EntityId) -> bool {
        self.entities.read().await.contains_key(&entity)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorldContract for World {
    async fn spawn_entity(&self) -> EcsResult<EntityId> {
        // Spawn a single entity without components
        let entities = self.spawn_batch(vec![vec![]]).await?;
        Ok(entities[0])
    }
    
    async fn despawn_batch(&self, entities: Vec<EntityId>) -> EcsResult<()> {
        self.despawn_batch(entities).await
    }
    
    async fn register_component<T: ComponentData>(&self) -> EcsResult<()> {
        self.register_component::<T>().await
    }
    
    async fn register_component_with_storage<T: ComponentData>(&self, storage_type: StorageType) -> EcsResult<()> {
        // Map to our internal method
        let component_id = T::component_id();
        self.registry.register::<T>().await?;
        
        if !self.storages.read().await.contains_key(&component_id) {
            let storage = handle(match storage_type {
                StorageType::Dense => ComponentStorage::new_dense(component_id.clone()),
                StorageType::Sparse | StorageType::Pooled => ComponentStorage::new_sparse(component_id.clone()),
            });
            self.storages.write().await.insert(component_id, storage);
        }
        
        Ok(())
    }
    
    async fn has_component(&self, entity: EntityId, component_id: ComponentId) -> bool {
        self.has_component(entity, component_id).await
    }
    
    async fn get_component<T: ComponentData>(&self, entity: EntityId) -> EcsResult<T> {
        self.get_component::<T>(entity).await
    }
    
    async fn query_entities(&self, required: Vec<ComponentId>, excluded: Vec<ComponentId>) -> EcsResult<Vec<EntityId>> {
        let mut query = self.query();
        for component_id in required {
            query = query.with_component(component_id);
        }
        for component_id in excluded {
            query = query.without_component(component_id);
        }
        self.execute_query(&query).await
    }
    
    async fn update(&self, delta_time: f32) -> EcsResult<()> {
        self.update(delta_time).await
    }
    
    async fn publish(&self, channel: playground_core_ecs::ChannelId, message: Bytes) -> EcsResult<()> {
        self.publish(channel, message).await
    }
    
    async fn subscribe(&self, channel: playground_core_ecs::ChannelId, handler_id: String) -> EcsResult<()> {
        self.subscribe(channel, handler_id).await
    }
    
    async fn unsubscribe(&self, channel: playground_core_ecs::ChannelId, handler_id: &str) -> EcsResult<()> {
        self.unsubscribe(channel, handler_id).await
    }
    
    async fn run_gc(&self) -> EcsResult<usize> {
        self.run_gc().await
    }
    
    async fn is_empty(&self) -> bool {
        self.is_empty().await
    }
}

#[async_trait]
impl WorldCommandHandler for World {
    async fn handle_command(&self, command: WorldCommand) {
        match command {
            WorldCommand::SpawnEntity { response } => {
                let result = self.spawn_entity().await;
                let _ = response.send(result);
            }
            WorldCommand::SpawnBatch { count, response } => {
                let bundles = vec![vec![]; count]; // Empty bundles
                let result = self.spawn_batch(bundles).await;
                let _ = response.send(result);
            }
            WorldCommand::DespawnEntity { entity, response } => {
                let result = self.despawn_batch(vec![entity]).await;
                let _ = response.send(result);
            }
            WorldCommand::DespawnBatch { entities, response } => {
                let result = self.despawn_batch(entities).await;
                let _ = response.send(result);
            }
            WorldCommand::QueryEntities { required, excluded, response } => {
                let result = self.query_entities(required, excluded).await;
                let _ = response.send(result);
            }
            WorldCommand::GetComponent { entity, component_id, response } => {
                let result = self.get_component_bytes(entity, component_id).await;
                let _ = response.send(result);
            }
            WorldCommand::SetComponent { entity, component_id, data, response } => {
                let result = self.set_component_bytes(entity, component_id, data).await;
                let _ = response.send(result);
            }
            WorldCommand::AddComponent { entity, component_id, data, response } => {
                let result = self.add_component_bytes(entity, component_id, data).await;
                let _ = response.send(result);
            }
            WorldCommand::RemoveComponent { entity, component_id, response } => {
                let result = self.remove_component(entity, component_id).await;
                let _ = response.send(result);
            }
            WorldCommand::HasComponent { entity, component_id, response } => {
                let result = self.has_component(entity, component_id).await;
                let _ = response.send(result);
            }
            WorldCommand::IsAlive { entity, response } => {
                let result = self.is_alive(entity).await;
                let _ = response.send(result);
            }
        }
    }
}