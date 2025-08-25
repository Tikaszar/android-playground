use std::marker::PhantomData;
use std::collections::HashMap;
use async_trait::async_trait;
use playground_core_types::{Handle, Shared};
use crate::entity::EntityId;
use crate::component::{ComponentData, ComponentId};
use crate::storage::{ComponentStorage, Storage};
use crate::error::{EcsError, EcsResult};

#[async_trait]
pub trait Query: Send + Sync {
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> EcsResult<Vec<EntityId>>;
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> bool;
}

pub struct ComponentIdQuery {
    component_id: ComponentId,
    include: bool, // true for with, false for without
}

impl ComponentIdQuery {
    pub fn new(component_id: ComponentId) -> Self {
        Self { component_id, include: true }
    }
    
    pub fn new_exclude(component_id: ComponentId) -> Self {
        Self { component_id, include: false }
    }
}

#[async_trait]
impl Query for ComponentIdQuery {
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        if self.include {
            if let Some(storage) = storages.read().await.get(&self.component_id) {
                Ok(storage.entities().await)
            } else {
                Ok(Vec::new())
            }
        } else {
            Err(EcsError::QueryError("Cannot execute 'without' query standalone".into()))
        }
    }
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> bool {
        if let Some(storage) = storages.read().await.get(&self.component_id) {
            let contains = storage.contains(entity).await;
            if self.include { contains } else { !contains }
        } else {
            !self.include
        }
    }
}

pub struct WithComponent<T: ComponentData> {
    _phantom: PhantomData<T>,
}

impl<T: ComponentData> WithComponent<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: ComponentData> Query for WithComponent<T> {
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        let component_id = T::component_id();
        
        if let Some(storage) = storages.read().await.get(&component_id) {
            Ok(storage.entities().await)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> bool {
        let component_id = T::component_id();
        
        if let Some(storage) = storages.read().await.get(&component_id) {
            storage.contains(entity).await
        } else {
            false
        }
    }
}

pub struct WithoutComponent<T: ComponentData> {
    _phantom: PhantomData<T>,
}

impl<T: ComponentData> WithoutComponent<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: ComponentData> Query for WithoutComponent<T> {
    async fn execute(&self, _storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        Err(EcsError::QueryError("WithoutComponent cannot be executed standalone".into()))
    }
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> bool {
        let component_id = T::component_id();
        
        if let Some(storage) = storages.read().await.get(&component_id) {
            !storage.contains(entity).await
        } else {
            true
        }
    }
}

// AndQuery uses component IDs directly instead of nested queries
pub struct AndQuery {
    component_ids: Vec<ComponentId>,
    include: Vec<bool>, // true for with, false for without
}

impl AndQuery {
    pub fn new() -> Self {
        Self { 
            component_ids: Vec::new(),
            include: Vec::new(),
        }
    }
    
    pub fn with(mut self, component_id: ComponentId) -> Self {
        self.component_ids.push(component_id);
        self.include.push(true);
        self
    }
    
    pub fn without(mut self, component_id: ComponentId) -> Self {
        self.component_ids.push(component_id);
        self.include.push(false);
        self
    }
}

#[async_trait]
impl Query for AndQuery {
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        if self.component_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // Start with entities from the first "with" component
        let mut result = Vec::new();
        let mut found_first = false;
        
        for (i, component_id) in self.component_ids.iter().enumerate() {
            if self.include[i] && !found_first {
                // First "with" component - get all its entities
                if let Some(storage) = storages.read().await.get(component_id) {
                    result = storage.entities().await;
                    found_first = true;
                }
            }
        }
        
        // If no "with" components, can't execute
        if !found_first {
            return Err(EcsError::QueryError("AndQuery needs at least one 'with' component".into()));
        }
        
        // Filter by remaining components
        let mut filtered = Vec::new();
        for entity in result {
            if self.matches(entity, storages).await {
                filtered.push(entity);
            }
        }
        result = filtered;
        
        Ok(result)
    }
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> bool {
        for (i, component_id) in self.component_ids.iter().enumerate() {
            let has_component = if let Some(storage) = storages.read().await.get(component_id) {
                storage.contains(entity).await
            } else {
                false
            };
            
            if self.include[i] != has_component {
                return false;
            }
        }
        true
    }
}

// OrQuery removed - can't use dyn, use AndQuery with inverted logic instead

pub struct QueryBuilder {
    component_ids: Vec<ComponentId>,
    include: Vec<bool>, // true for with, false for without
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            component_ids: Vec::new(),
            include: Vec::new(),
        }
    }
    
    pub fn with_component(mut self, component_id: ComponentId) -> Self {
        self.component_ids.push(component_id);
        self.include.push(true);
        self
    }
    
    pub fn without_component(mut self, component_id: ComponentId) -> Self {
        self.component_ids.push(component_id);
        self.include.push(false);
        self
    }
    
    pub fn build(self) -> AndQuery {
        let mut query = AndQuery::new();
        for (i, component_id) in self.component_ids.into_iter().enumerate() {
            if self.include[i] {
                query = query.with(component_id);
            } else {
                query = query.without(component_id);
            }
        }
        query
    }
}

struct EmptyQuery;

#[async_trait]
impl Query for EmptyQuery {
    async fn execute(&self, _storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        Ok(Vec::new())
    }
    
    async fn matches(&self, _entity: EntityId, _storages: &Shared<HashMap<ComponentId, Handle<ComponentStorage>>>) -> bool {
        true
    }
}

// CachedQuery removed - can't use dyn, caching should be done at a higher level