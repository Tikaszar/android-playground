//! World implementation - all the logic for ECS operations
//! 
//! This implements all the actual ECS logic that operates on the World data structure
//! from core/ecs. Think of this as implementing the methods of an abstract class.

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use playground_core_ecs::{
    World, EntityId, Generation, Component, ComponentId,
    CoreResult, CoreError, entity_not_found, component_not_found
};
use playground_core_types::Handle;

/// Implementation of all World operations
pub struct WorldImpl;

impl WorldImpl {
    /// Spawn a new entity in the World
    pub async fn spawn_entity(world: &Handle<World>) -> CoreResult<EntityId> {
        let id = world.next_entity_id.fetch_add(1, Ordering::SeqCst);
        let entity_id = EntityId::new(id);

        // New entities start at Generation 1
        let mut entities = world.entities.write().await;
        entities.insert(entity_id, Generation::new());
        drop(entities);

        let mut components = world.components.write().await;
        components.insert(entity_id, HashMap::new());

        Ok(entity_id)
    }
    
    /// Despawn an entity from the World
    pub async fn despawn_entity(world: &Handle<World>, entity: EntityId) -> CoreResult<()> {
        let mut entities = world.entities.write().await;

        // Increment generation to mark as invalid (for future reuse)
        if let Some(generation) = entities.get_mut(&entity) {
            generation.increment();
        } else {
            return Err(entity_not_found(entity));
        }
        drop(entities);

        // Remove components
        let mut components = world.components.write().await;
        components.remove(&entity);

        Ok(())
    }
    
    /// Despawn multiple entities
    pub async fn despawn_batch(world: &Handle<World>, entity_list: Vec<EntityId>) -> CoreResult<()> {
        let mut entities = world.entities.write().await;
        let mut components = world.components.write().await;
        
        for entity in entity_list {
            entities.remove(&entity);
            components.remove(&entity);
        }
        
        Ok(())
    }
    
    /// Check if an entity exists
    pub async fn has_entity(world: &Handle<World>, entity: EntityId) -> bool {
        let entities = world.entities.read().await;
        entities.contains_key(&entity)
    }
    
    /// Add a component to an entity
    pub async fn add_component(world: &Handle<World>, entity: EntityId, component: Component) -> CoreResult<()> {
        // Check entity exists
        if !Self::has_entity(world, entity).await {
            return Err(entity_not_found(entity));
        }
        
        let mut components = world.components.write().await;
        if let Some(entity_components) = components.get_mut(&entity) {
            entity_components.insert(component.component_id, component);
        }
        
        Ok(())
    }
    
    /// Remove a component from an entity
    pub async fn remove_component(world: &Handle<World>, entity: EntityId, component_id: ComponentId) -> CoreResult<()> {
        let mut components = world.components.write().await;
        if let Some(entity_components) = components.get_mut(&entity) {
            entity_components.remove(&component_id);
        }
        Ok(())
    }
    
    /// Get a component from an entity
    pub async fn get_component(world: &Handle<World>, entity: EntityId, component_id: ComponentId) -> CoreResult<Component> {
        let components = world.components.read().await;
        components.get(&entity)
            .and_then(|entity_components| entity_components.get(&component_id))
            .cloned()
            .ok_or(component_not_found(entity, component_id))
    }
    
    /// Check if an entity has a component
    pub async fn has_component(world: &Handle<World>, entity: EntityId, component_id: ComponentId) -> bool {
        let components = world.components.read().await;
        components.get(&entity)
            .map(|entity_components| entity_components.contains_key(&component_id))
            .unwrap_or(false)
    }
    
    /// Query entities with specific components
    pub async fn query(world: &Handle<World>, required: Vec<ComponentId>, excluded: Vec<ComponentId>) -> CoreResult<Vec<EntityId>> {
        let entities = world.entities.read().await;
        let components = world.components.read().await;
        
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
    
    /// Get all entities
    pub async fn all_entities(world: &Handle<World>) -> Vec<EntityId> {
        let entities = world.entities.read().await;
        entities.keys().copied().collect()
    }
    
    /// Get all components for an entity
    pub async fn get_components(world: &Handle<World>, entity: EntityId) -> CoreResult<Vec<Component>> {
        let components = world.components.read().await;
        let entity_components = components.get(&entity)
            .ok_or_else(|| entity_not_found(entity))?;

        Ok(entity_components.values().cloned().collect())
    }

    /// Validate an entity's existence and generation
    pub async fn validate_entity(world: &Handle<World>, id: EntityId, generation: Generation) -> CoreResult<bool> {
        let entities = world.entities.read().await;

        match entities.get(&id) {
            Some(current_gen) => Ok(*current_gen == generation),
            None => Ok(false),
        }
    }
}