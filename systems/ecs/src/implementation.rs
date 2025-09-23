//! Core ECS implementation
//! 
//! This provides the actual ECS functionality - entity management, component storage,
//! system scheduling, etc.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use playground_core_types::{Shared, shared, CoreResult, CoreError, EntityIdError};
use playground_core_ecs::{
    EntityId, Generation, Component, ComponentId,
    Query, QueryResult, entity_not_found, component_not_found
};

/// The actual ECS implementation
pub struct EcsImplementation {
    // Entity management
    next_entity_id: AtomicU32,
    entities: Shared<HashMap<EntityId, Generation>>,
    
    // Component storage - maps entity to components
    components: Shared<HashMap<EntityId, HashMap<ComponentId, Component>>>,
    
    // Stats
    entity_count: AtomicU32,
    component_count: AtomicU32,
}

impl EcsImplementation {
    pub fn new() -> Self {
        Self {
            next_entity_id: AtomicU32::new(1),
            entities: shared(HashMap::new()),
            components: shared(HashMap::new()),
            entity_count: AtomicU32::new(0),
            component_count: AtomicU32::new(0),
        }
    }
    
    /// Spawn a new entity
    pub async fn spawn_entity(&self) -> CoreResult<EntityId> {
        let index = self.next_entity_id.fetch_add(1, Ordering::SeqCst);
        let entity = EntityId::new(index);
        
        let mut entities = self.entities.write().await;
        entities.insert(entity, Generation::new());
        
        let mut components = self.components.write().await;
        components.insert(entity, HashMap::new());
        
        self.entity_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(entity)
    }
    
    /// Despawn an entity
    pub async fn despawn_entity(&self, entity: EntityId) -> CoreResult<()> {
        let mut entities = self.entities.write().await;
        if entities.remove(&entity).is_none() {
            return Err(entity_not_found(entity));
        }
        
        let mut components = self.components.write().await;
        if let Some(entity_components) = components.remove(&entity) {
            let count = entity_components.len() as u32;
            self.component_count.fetch_sub(count, Ordering::Relaxed);
        }
        
        self.entity_count.fetch_sub(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Add a component to an entity
    pub async fn add_component(&self, entity: EntityId, component: Component) -> CoreResult<()> {
        let entities = self.entities.read().await;
        if !entities.contains_key(&entity) {
            return Err(entity_not_found(entity));
        }
        drop(entities);
        
        let mut components = self.components.write().await;
        let entity_components = components.get_mut(&entity)
            .ok_or(entity_not_found(entity))?;
        
        let is_new = entity_components.insert(component.component_id, component).is_none();
        if is_new {
            self.component_count.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(())
    }
    
    /// Remove a component from an entity
    pub async fn remove_component(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<()> {
        let mut components = self.components.write().await;
        let entity_components = components.get_mut(&entity)
            .ok_or(entity_not_found(entity))?;
        
        if entity_components.remove(&component_id).is_some() {
            self.component_count.fetch_sub(1, Ordering::Relaxed);
            Ok(())
        } else {
            Err(component_not_found(entity, component_id))
        }
    }
    
    /// Get a component from an entity
    pub async fn get_component(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<Component> {
        let components = self.components.read().await;
        let entity_components = components.get(&entity)
            .ok_or(entity_not_found(entity))?;
        
        entity_components.get(&component_id)
            .cloned()
            .ok_or(component_not_found(entity, component_id))
    }
    
    /// Query entities with specific components
    pub async fn query(&self, query: Query) -> CoreResult<QueryResult> {
        let components = self.components.read().await;
        let mut results = Vec::new();
        
        for (entity, entity_components) in components.iter() {
            // Check if entity matches query
            let component_ids: Vec<ComponentId> = entity_components.keys().copied().collect();
            if query.matches(&component_ids) {
                results.push(*entity);
                
                // Check limit
                if let Some(limit) = query.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }
        
        let total_count = results.len();
        Ok(QueryResult::new(results, total_count))
    }
    
    /// Get statistics
    pub fn stats(&self) -> (u32, u32) {
        (
            self.entity_count.load(Ordering::Relaxed),
            self.component_count.load(Ordering::Relaxed)
        )
    }
}