//! Query system for the unified ECS

use std::collections::{HashMap, HashSet};
use async_trait::async_trait;
use playground_core_types::{Handle, Shared};
use playground_core_ecs::{EntityId, ComponentId, EcsResult, EcsError, Query as QueryTrait};
use crate::storage::ComponentStorage;

/// Query builder for finding entities with specific components
pub struct QueryBuilder {
    required: Vec<ComponentId>,
    excluded: Vec<ComponentId>,
    optional: Vec<ComponentId>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            required: Vec::new(),
            excluded: Vec::new(),
            optional: Vec::new(),
        }
    }
    
    /// Add a required component to the query (NO TURBOFISH!)
    pub fn with_component(mut self, component_id: ComponentId) -> Self {
        self.required.push(component_id);
        self
    }
    
    /// Add an excluded component to the query
    pub fn without_component(mut self, component_id: ComponentId) -> Self {
        self.excluded.push(component_id);
        self
    }
    
    /// Add an optional component to the query
    pub fn optional_component(mut self, component_id: ComponentId) -> Self {
        self.optional.push(component_id);
        self
    }
    
    /// Execute the query against component storages
    pub async fn execute(&self, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        if self.required.is_empty() {
            return Ok(Vec::new());
        }
        
        // Get all entities that have the required components
        let storages_guard = storages.read().await;
        
        // Start with entities from the first required component
        let first_storage = storages_guard.get(&self.required[0])
            .ok_or_else(|| EcsError::ComponentNotRegistered(self.required[0].clone()))?;
        
        let mut result: HashSet<EntityId> = first_storage.get_dirty().await
            .into_iter()
            .collect();
        
        // If no dirty entities, get all entities
        if result.is_empty() {
            let all_entities = self.get_all_entities_with_component(&self.required[0], &storages_guard).await?;
            result = all_entities.into_iter().collect();
        }
        
        // Filter by remaining required components
        for component_id in self.required.iter().skip(1) {
            let storage = storages_guard.get(component_id)
                .ok_or_else(|| EcsError::ComponentNotRegistered(component_id.clone()))?;
            
            let mut matching = HashSet::new();
            for entity in &result {
                if storage.contains(*entity).await {
                    matching.insert(*entity);
                }
            }
            result = matching;
            
            if result.is_empty() {
                break;
            }
        }
        
        // Filter out excluded components
        for component_id in &self.excluded {
            if let Some(storage) = storages_guard.get(component_id) {
                let mut filtered = HashSet::new();
                for entity in &result {
                    if !storage.contains(*entity).await {
                        filtered.insert(*entity);
                    }
                }
                result = filtered;
            }
        }
        
        Ok(result.into_iter().collect())
    }
    
    /// Get all entities that have a specific component
    async fn get_all_entities_with_component(
        &self, 
        component_id: &ComponentId,
        storages: &HashMap<ComponentId, Handle<ComponentStorage>>
    ) -> EcsResult<Vec<EntityId>> {
        let _storage = storages.get(component_id)
            .ok_or_else(|| EcsError::ComponentNotRegistered(component_id.clone()))?;
        
        // For now, we'll check all entities - in a real implementation
        // we'd have a better way to iterate entities
        let entities = Vec::new();
        
        // This is a placeholder - we need a better way to get all entities
        // In the actual implementation, storage should have an entities() method
        // For now, we'll return empty
        Ok(entities)
    }
    
    /// Check if the query matches an entity
    pub async fn matches(
        &self, 
        entity: EntityId,
        storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>
    ) -> bool {
        let storages_guard = storages.read().await;
        
        // Check required components
        for component_id in &self.required {
            if let Some(storage) = storages_guard.get(component_id) {
                if !storage.contains(entity).await {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check excluded components
        for component_id in &self.excluded {
            if let Some(storage) = storages_guard.get(component_id) {
                if storage.contains(entity).await {
                    return false;
                }
            }
        }
        
        true
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QueryTrait for QueryBuilder {
    fn with_component(mut self, component_id: ComponentId) -> Self where Self: Sized {
        self.required.push(component_id);
        self
    }
    
    fn without_component(mut self, component_id: ComponentId) -> Self where Self: Sized {
        self.excluded.push(component_id);
        self
    }
    
    fn optional_component(mut self, component_id: ComponentId) -> Self where Self: Sized {
        self.optional.push(component_id);
        self
    }
    
    async fn execute(&self) -> EcsResult<Vec<EntityId>> {
        // This needs access to storages, which we don't have here
        // This is a design issue - Query needs context to execute
        Err(EcsError::QueryFailed("Query execution requires World context".to_string()))
    }
    
    async fn matches(&self, _entity: EntityId) -> bool {
        // Same issue - needs World context
        false
    }
}

/// Query result iterator
pub struct QueryResult {
    entities: Vec<EntityId>,
    index: usize,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(entities: Vec<EntityId>) -> Self {
        Self {
            entities,
            index: 0,
        }
    }
    
    /// Get the total count of matching entities
    pub fn count(&self) -> usize {
        self.entities.len()
    }
    
    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
    
    /// Get all entities as a vec
    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }
}

impl Iterator for QueryResult {
    type Item = EntityId;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entities.len() {
            let entity = self.entities[self.index];
            self.index += 1;
            Some(entity)
        } else {
            None
        }
    }
}