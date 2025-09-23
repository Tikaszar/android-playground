//! ECS operations that are performed on the World data
//! 
//! These operations work with the World's data fields from core/ecs

use playground_core_ecs::{
    World, EntityId, Generation, Component, ComponentId,
    Query, QueryResult, get_world,
    entity_not_found, component_not_found
};
use playground_core_types::{CoreResult, CoreError};
use std::sync::atomic::Ordering;

/// Spawn a new entity in the World
pub async fn spawn_entity(world: &World) -> CoreResult<EntityId> {
    let index = world.next_entity_id.fetch_add(1, Ordering::SeqCst);
    let entity = EntityId::new(index);
    
    let mut entities = world.entities.write().await;
    entities.insert(entity, Generation::new());
    
    let mut components = world.components.write().await;
    components.insert(entity, std::collections::HashMap::new());
    
    Ok(entity)
}

/// Despawn an entity from the World
pub async fn despawn_entity(world: &World, entity: EntityId) -> CoreResult<()> {
    let mut entities = world.entities.write().await;
    if entities.remove(&entity).is_none() {
        return Err(entity_not_found(entity));
    }
    
    let mut components = world.components.write().await;
    components.remove(&entity);
    
    Ok(())
}

/// Add a component to an entity
pub async fn add_component(world: &World, entity: EntityId, component: Component) -> CoreResult<()> {
    // Check entity exists
    let entities = world.entities.read().await;
    if !entities.contains_key(&entity) {
        return Err(entity_not_found(entity));
    }
    drop(entities);
    
    let mut components = world.components.write().await;
    if let Some(entity_components) = components.get_mut(&entity) {
        entity_components.insert(component.component_id, component);
    } else {
        return Err(entity_not_found(entity));
    }
    
    Ok(())
}

/// Remove a component from an entity
pub async fn remove_component(world: &World, entity: EntityId, component_id: ComponentId) -> CoreResult<()> {
    let mut components = world.components.write().await;
    let entity_components = components.get_mut(&entity)
        .ok_or_else(|| entity_not_found(entity))?;
    
    if entity_components.remove(&component_id).is_none() {
        return Err(component_not_found(entity, component_id));
    }
    
    Ok(())
}

/// Get a component from an entity
pub async fn get_component(world: &World, entity: EntityId, component_id: ComponentId) -> CoreResult<Component> {
    let components = world.components.read().await;
    let entity_components = components.get(&entity)
        .ok_or_else(|| entity_not_found(entity))?;
    
    entity_components.get(&component_id)
        .cloned()
        .ok_or_else(|| component_not_found(entity, component_id))
}

/// Query entities with specific components
pub async fn query(world: &World, query: Query) -> CoreResult<QueryResult> {
    let components = world.components.read().await;
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

/// Check if an entity exists
pub async fn has_entity(world: &World, entity: EntityId) -> bool {
    let entities = world.entities.read().await;
    entities.contains_key(&entity)
}

/// Check if an entity has a component
pub async fn has_component(world: &World, entity: EntityId, component_id: ComponentId) -> bool {
    let components = world.components.read().await;
    components.get(&entity)
        .map(|entity_components| entity_components.contains_key(&component_id))
        .unwrap_or(false)
}

/// Get all entities
pub async fn all_entities(world: &World) -> Vec<EntityId> {
    let entities = world.entities.read().await;
    entities.keys().copied().collect()
}

/// Get all components for an entity
pub async fn get_components(world: &World, entity: EntityId) -> CoreResult<Vec<Component>> {
    let components = world.components.read().await;
    let entity_components = components.get(&entity)
        .ok_or_else(|| entity_not_found(entity))?;
    
    Ok(entity_components.values().cloned().collect())
}