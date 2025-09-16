//! Concrete World implementation for the ECS
//! 
//! The World is the central container for all entities, components, and systems.
//! It also contains the VTable for dispatching to system implementations.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use playground_core_types::{Handle, handle, Shared, shared};
use tokio::sync::mpsc;
use crate::{
    Component, ComponentId, EntityId, Generation,
    CoreError, CoreResult, VTable,
};

/// Commands for entity operations
pub enum EntityCommand {
    Spawn(tokio::sync::oneshot::Sender<CoreResult<EntityId>>),
    Despawn(EntityId, tokio::sync::oneshot::Sender<CoreResult<()>>),
    DespawnBatch(Vec<EntityId>, tokio::sync::oneshot::Sender<CoreResult<()>>),
    HasEntity(EntityId, tokio::sync::oneshot::Sender<bool>),
}

/// Commands for component operations
pub enum ComponentCommand {
    Add {
        entity: EntityId,
        component: Component,
        response: tokio::sync::oneshot::Sender<CoreResult<()>>,
    },
    Remove {
        entity: EntityId,
        component_id: ComponentId,
        response: tokio::sync::oneshot::Sender<CoreResult<()>>,
    },
    Get {
        entity: EntityId,
        component_id: ComponentId,
        response: tokio::sync::oneshot::Sender<CoreResult<Component>>,
    },
    Has {
        entity: EntityId,
        component_id: ComponentId,
        response: tokio::sync::oneshot::Sender<bool>,
    },
}

/// The concrete World struct - NOT a trait!
pub struct World {
    /// Entity generation tracking
    entities: Shared<HashMap<EntityId, Generation>>,
    
    /// Component storage: entity -> component_id -> component
    components: Shared<HashMap<EntityId, HashMap<ComponentId, Component>>>,
    
    /// The VTable for system dispatch
    pub vtable: VTable,
    
    /// Next entity ID
    next_entity_id: AtomicU32,
    
    /// Channel for entity commands
    entity_sender: mpsc::Sender<EntityCommand>,
    
    /// Channel for component commands
    component_sender: mpsc::Sender<ComponentCommand>,
}

impl World {
    /// Create a new World instance
    pub fn new() -> Handle<Self> {
        // Create command channels
        let (entity_tx, mut entity_rx) = mpsc::channel::<EntityCommand>(100);
        let (component_tx, mut component_rx) = mpsc::channel::<ComponentCommand>(100);
        
        let world = handle(Self {
            entities: shared(HashMap::new()),
            components: shared(HashMap::new()),
            vtable: VTable::new(),
            next_entity_id: AtomicU32::new(1),
            entity_sender: entity_tx,
            component_sender: component_tx,
        });
        
        // Spawn entity command handler
        let world_clone = world.clone();
        tokio::spawn(async move {
            while let Some(cmd) = entity_rx.recv().await {
                match cmd {
                    EntityCommand::Spawn(response) => {
                        let id = world_clone.spawn_entity_impl().await;
                        let _ = response.send(id);
                    }
                    EntityCommand::Despawn(entity, response) => {
                        let result = world_clone.despawn_entity_impl(entity).await;
                        let _ = response.send(result);
                    }
                    EntityCommand::DespawnBatch(entities, response) => {
                        let result = world_clone.despawn_batch_impl(entities).await;
                        let _ = response.send(result);
                    }
                    EntityCommand::HasEntity(entity, response) => {
                        let exists = world_clone.has_entity_impl(entity).await;
                        let _ = response.send(exists);
                    }
                }
            }
        });
        
        // Spawn component command handler
        let world_clone = world.clone();
        tokio::spawn(async move {
            while let Some(cmd) = component_rx.recv().await {
                match cmd {
                    ComponentCommand::Add { entity, component, response } => {
                        let result = world_clone.add_component_impl(entity, component).await;
                        let _ = response.send(result);
                    }
                    ComponentCommand::Remove { entity, component_id, response } => {
                        let result = world_clone.remove_component_impl(entity, component_id).await;
                        let _ = response.send(result);
                    }
                    ComponentCommand::Get { entity, component_id, response } => {
                        let result = world_clone.get_component_impl(entity, component_id).await;
                        let _ = response.send(result);
                    }
                    ComponentCommand::Has { entity, component_id, response } => {
                        let has = world_clone.has_component_impl(entity, component_id).await;
                        let _ = response.send(has);
                    }
                }
            }
        });
        
        world
    }
    
    // Public API methods (these delegate to command channels)
    
    /// Spawn a new entity
    pub async fn spawn_entity(&self) -> CoreResult<EntityId> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.entity_sender.send(EntityCommand::Spawn(tx)).await
            .map_err(|_| CoreError::SendError)?;
        rx.await.map_err(|_| CoreError::ReceiveError)?
    }
    
    /// Despawn an entity
    pub async fn despawn_entity(&self, entity: EntityId) -> CoreResult<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.entity_sender.send(EntityCommand::Despawn(entity, tx)).await
            .map_err(|_| CoreError::SendError)?;
        rx.await.map_err(|_| CoreError::ReceiveError)?
    }
    
    /// Despawn multiple entities
    pub async fn despawn_batch(&self, entities: Vec<EntityId>) -> CoreResult<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.entity_sender.send(EntityCommand::DespawnBatch(entities, tx)).await
            .map_err(|_| CoreError::SendError)?;
        rx.await.map_err(|_| CoreError::ReceiveError)?
    }
    
    /// Check if an entity exists
    pub async fn has_entity(&self, entity: EntityId) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();
        if self.entity_sender.send(EntityCommand::HasEntity(entity, tx)).await.is_err() {
            return false;
        }
        rx.await.unwrap_or(false)
    }
    
    /// Add a component to an entity
    pub async fn add_component(&self, entity: EntityId, component: Component) -> CoreResult<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.component_sender.send(ComponentCommand::Add { entity, component, response: tx }).await
            .map_err(|_| CoreError::SendError)?;
        rx.await.map_err(|_| CoreError::ReceiveError)?
    }
    
    /// Remove a component from an entity
    pub async fn remove_component(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.component_sender.send(ComponentCommand::Remove { entity, component_id, response: tx }).await
            .map_err(|_| CoreError::SendError)?;
        rx.await.map_err(|_| CoreError::ReceiveError)?
    }
    
    /// Get a component from an entity
    pub async fn get_component(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<Component> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.component_sender.send(ComponentCommand::Get { entity, component_id, response: tx }).await
            .map_err(|_| CoreError::SendError)?;
        rx.await.map_err(|_| CoreError::ReceiveError)?
    }
    
    /// Check if an entity has a component
    pub async fn has_component(&self, entity: EntityId, component_id: ComponentId) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();
        if self.component_sender.send(ComponentCommand::Has { entity, component_id, response: tx }).await.is_err() {
            return false;
        }
        rx.await.unwrap_or(false)
    }
    
    // Implementation methods (called by command handlers)
    
    async fn spawn_entity_impl(&self) -> CoreResult<EntityId> {
        let id = self.next_entity_id.fetch_add(1, Ordering::SeqCst);
        let entity_id = EntityId(id);
        
        let mut entities = self.entities.write().await;
        entities.insert(entity_id, Generation(0));
        
        let mut components = self.components.write().await;
        components.insert(entity_id, HashMap::new());
        
        Ok(entity_id)
    }
    
    async fn despawn_entity_impl(&self, entity: EntityId) -> CoreResult<()> {
        let mut entities = self.entities.write().await;
        let mut components = self.components.write().await;
        
        if entities.remove(&entity).is_none() {
            return Err(CoreError::EntityNotFound(entity));
        }
        
        components.remove(&entity);
        Ok(())
    }
    
    async fn despawn_batch_impl(&self, entity_list: Vec<EntityId>) -> CoreResult<()> {
        let mut entities = self.entities.write().await;
        let mut components = self.components.write().await;
        
        for entity in entity_list {
            entities.remove(&entity);
            components.remove(&entity);
        }
        
        Ok(())
    }
    
    async fn has_entity_impl(&self, entity: EntityId) -> bool {
        let entities = self.entities.read().await;
        entities.contains_key(&entity)
    }
    
    async fn add_component_impl(&self, entity: EntityId, component: Component) -> CoreResult<()> {
        // Check entity exists
        if !self.has_entity_impl(entity).await {
            return Err(CoreError::EntityNotFound(entity));
        }
        
        let mut components = self.components.write().await;
        if let Some(entity_components) = components.get_mut(&entity) {
            entity_components.insert(component.component_id, component);
        }
        
        Ok(())
    }
    
    async fn remove_component_impl(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<()> {
        let mut components = self.components.write().await;
        if let Some(entity_components) = components.get_mut(&entity) {
            entity_components.remove(&component_id);
        }
        Ok(())
    }
    
    async fn get_component_impl(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<Component> {
        let components = self.components.read().await;
        components.get(&entity)
            .and_then(|entity_components| entity_components.get(&component_id))
            .cloned()
            .ok_or(CoreError::ComponentNotFound(entity, component_id))
    }
    
    async fn has_component_impl(&self, entity: EntityId, component_id: ComponentId) -> bool {
        let components = self.components.read().await;
        components.get(&entity)
            .map(|entity_components| entity_components.contains_key(&component_id))
            .unwrap_or(false)
    }
    
    /// Query entities with specific components
    pub async fn query(&self, required: Vec<ComponentId>, excluded: Vec<ComponentId>) -> CoreResult<Vec<EntityId>> {
        let entities = self.entities.read().await;
        let components = self.components.read().await;
        
        let mut results = Vec::new();
        
        for (entity_id, _generation) in entities.iter() {
            if let Some(entity_components) = components.get(entity_id) {
                // Check all required components are present
                let has_required = required.iter().all(|comp_id| entity_components.contains_key(comp_id));
                
                // Check none of the excluded components are present
                let has_excluded = excluded.iter().any(|comp_id| entity_components.contains_key(comp_id));
                
                if has_required && !has_excluded {
                    results.push(*entity_id);
                }
            }
        }
        
        Ok(results)
    }
}