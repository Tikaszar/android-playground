use std::marker::PhantomData;
use std::collections::HashMap;
use async_trait::async_trait;
use playground_core_types::{Shared, shared};
use crate::entity::EntityId;
use crate::component::{Component, ComponentId};
use crate::storage::ComponentStorage;
use crate::error::{EcsError, EcsResult};

#[async_trait]
pub trait Query: Send + Sync {
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> EcsResult<Vec<EntityId>>;
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> bool;
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
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
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
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> bool {
        if let Some(storage) = storages.read().await.get(&self.component_id) {
            let contains = storage.contains(entity).await;
            if self.include { contains } else { !contains }
        } else {
            !self.include
        }
    }
}

pub struct WithComponent<T: Component> {
    _phantom: PhantomData<T>,
}

impl<T: Component> WithComponent<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: Component> Query for WithComponent<T> {
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        let component_id = T::component_id();
        
        if let Some(storage) = storages.read().await.get(&component_id) {
            Ok(storage.entities().await)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> bool {
        let component_id = T::component_id();
        
        if let Some(storage) = storages.read().await.get(&component_id) {
            storage.contains(entity).await
        } else {
            false
        }
    }
}

pub struct WithoutComponent<T: Component> {
    _phantom: PhantomData<T>,
}

impl<T: Component> WithoutComponent<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: Component> Query for WithoutComponent<T> {
    async fn execute(&self, _storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        Err(EcsError::QueryError("WithoutComponent cannot be executed standalone".into()))
    }
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> bool {
        let component_id = T::component_id();
        
        if let Some(storage) = storages.read().await.get(&component_id) {
            !storage.contains(entity).await
        } else {
            true
        }
    }
}

pub struct AndQuery {
    queries: Vec<Box<dyn Query>>,
}

impl AndQuery {
    pub fn new(queries: Vec<Box<dyn Query>>) -> Self {
        Self { queries }
    }
}

#[async_trait]
impl Query for AndQuery {
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        if self.queries.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut result = self.queries[0].execute(storages).await?;
        
        for query in &self.queries[1..] {
            let mut filtered = Vec::new();
            for entity in result {
                if query.matches(entity, storages).await {
                    filtered.push(entity);
                }
            }
            result = filtered;
        }
        
        Ok(result)
    }
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> bool {
        for query in &self.queries {
            if !query.matches(entity, storages).await {
                return false;
            }
        }
        true
    }
}

pub struct OrQuery {
    queries: Vec<Box<dyn Query>>,
}

impl OrQuery {
    pub fn new(queries: Vec<Box<dyn Query>>) -> Self {
        Self { queries }
    }
}

#[async_trait]
impl Query for OrQuery {
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        for query in &self.queries {
            let entities = query.execute(storages).await?;
            for entity in entities {
                if seen.insert(entity) {
                    result.push(entity);
                }
            }
        }
        
        Ok(result)
    }
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> bool {
        for query in &self.queries {
            if query.matches(entity, storages).await {
                return true;
            }
        }
        false
    }
}

pub struct QueryBuilder {
    queries: Vec<Box<dyn Query>>,
    exclude: Vec<Box<dyn Query>>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
            exclude: Vec::new(),
        }
    }
    
    pub fn with_component(mut self, component_id: ComponentId) -> Self {
        // We'll need to create a generic query for any component ID
        self.queries.push(Box::new(ComponentIdQuery::new(component_id)));
        self
    }
    
    pub fn without_component(mut self, component_id: ComponentId) -> Self {
        self.exclude.push(Box::new(ComponentIdQuery::new_exclude(component_id)));
        self
    }
    
    pub fn build(self) -> Box<dyn Query> {
        let mut all_queries = self.queries;
        all_queries.extend(self.exclude);
        
        if all_queries.is_empty() {
            Box::new(EmptyQuery)
        } else if all_queries.len() == 1 {
            all_queries.into_iter().next().unwrap()
        } else {
            Box::new(AndQuery::new(all_queries))
        }
    }
}

struct EmptyQuery;

#[async_trait]
impl Query for EmptyQuery {
    async fn execute(&self, _storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        Ok(Vec::new())
    }
    
    async fn matches(&self, _entity: EntityId, _storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> bool {
        true
    }
}

pub struct CachedQuery {
    inner: Box<dyn Query>,
    cache: Shared<Option<Vec<EntityId>>>,
}

impl CachedQuery {
    pub fn new(inner: Box<dyn Query>) -> Self {
        Self {
            inner,
            cache: shared(None),
        }
    }
    
    pub async fn invalidate(&self) {
        *self.cache.write().await = None;
    }
}

#[async_trait]
impl Query for CachedQuery {
    async fn execute(&self, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> EcsResult<Vec<EntityId>> {
        if let Some(cached) = self.cache.read().await.as_ref() {
            return Ok(cached.clone());
        }
        
        let result = self.inner.execute(storages).await?;
        *self.cache.write().await = Some(result.clone());
        Ok(result)
    }
    
    async fn matches(&self, entity: EntityId, storages: &Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>) -> bool {
        self.inner.matches(entity, storages).await
    }
}