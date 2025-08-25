use crate::entity::Entity;
use crate::storage::HybridStorage;
use crate::component::ComponentId;
use fnv::FnvHashMap;
use playground_core_types::{Shared, shared};

/// Query builder for ECS queries
pub struct QueryBuilder<'a> {
    storage: &'a HybridStorage,
    with_components: Vec<ComponentId>,
    without_components: Vec<ComponentId>,
    cached: bool,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(storage: &'a HybridStorage) -> Self {
        Self {
            storage,
            with_components: Vec::new(),
            without_components: Vec::new(),
            cached: false,
        }
    }
    
    pub fn with_component(mut self, component_id: ComponentId) -> Self {
        self.with_components.push(component_id);
        self
    }
    
    pub fn without_component(mut self, component_id: ComponentId) -> Self {
        self.without_components.push(component_id);
        self
    }
    
    pub fn cached(mut self) -> Self {
        self.cached = true;
        self
    }
    
    pub fn build(self) -> Query<'a> {
        let cache_id = if self.cached {
            Some(generate_cache_id(&self.with_components, &self.without_components))
        } else {
            None
        };
        
        Query {
            storage: self.storage,
            with_components: self.with_components,
            without_components: self.without_components,
            cached: self.cached,
            cache_id,
        }
    }
}

/// Query result iterator
pub struct Query<'a> {
    storage: &'a HybridStorage,
    with_components: Vec<ComponentId>,
    without_components: Vec<ComponentId>,
    cached: bool,
    cache_id: Option<u64>,
}

impl<'a> Query<'a> {
    pub async fn iter(&self) -> QueryIter<'a> {
        // If cached, try to get from cache
        if let Some(cache_id) = self.cache_id {
            if let Some(cached_entities) = QUERY_CACHE.get(cache_id).await {
                return QueryIter {
                    entities: cached_entities,
                    current: 0,
                    storage: self.storage,
                };
            }
        }
        
        // Build entity list
        let mut entities = Vec::new();
        
        // Get all entities from archetype storage
        for entity in self.storage.iter_archetype_entities().await {
            if self.matches_entity(entity) {
                entities.push(entity);
            }
        }
        
        // Cache if requested
        if let Some(cache_id) = self.cache_id {
            QUERY_CACHE.insert(cache_id, entities.clone()).await;
        }
        
        QueryIter {
            entities,
            current: 0,
            storage: self.storage,
        }
    }
    
    pub async fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(Entity),
    {
        for entity in self.iter().await {
            f(entity);
        }
    }
    
    pub async fn par_for_each<F>(&self, f: F)
    where
        F: Fn(Entity) + Send + Sync,
    {
        use rayon::prelude::*;
        let entities: Vec<_> = self.iter().await.collect();
        entities.par_iter().for_each(|&entity| f(entity));
    }
    
    fn matches_entity(&self, entity: Entity) -> bool {
        // Check all required components are present
        for &component_id in &self.with_components {
            if !self.has_component(entity, component_id) {
                return false;
            }
        }
        
        // Check no excluded components are present
        for &component_id in &self.without_components {
            if self.has_component(entity, component_id) {
                return false;
            }
        }
        
        true
    }
    
    fn has_component(&self, _entity: Entity, _component_id: ComponentId) -> bool {
        // This is simplified - real implementation would check both archetype and sparse
        false
    }
}

pub struct QueryIter<'a> {
    entities: Vec<Entity>,
    current: usize,
    storage: &'a HybridStorage,
}

impl<'a> Iterator for QueryIter<'a> {
    type Item = Entity;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.entities.len() {
            let entity = self.entities[self.current];
            self.current += 1;
            Some(entity)
        } else {
            None
        }
    }
}

// Query cache for frequently used queries
lazy_static::lazy_static! {
    static ref QUERY_CACHE: QueryCache = QueryCache::new();
}

struct QueryCache {
    cache: Shared<FnvHashMap<u64, Vec<Entity>>>,
}

impl QueryCache {
    fn new() -> Self {
        Self {
            cache: shared(FnvHashMap::default()),
        }
    }
    
    async fn get(&self, id: u64) -> Option<Vec<Entity>> {
        self.cache.read().await.get(&id).cloned()
    }
    
    async fn insert(&self, id: u64, entities: Vec<Entity>) {
        self.cache.write().await.insert(id, entities);
    }
    
    async fn invalidate(&self, id: u64) {
        self.cache.write().await.remove(&id);
    }
    
    async fn clear(&self) {
        self.cache.write().await.clear();
    }
}

fn generate_cache_id(with: &[ComponentId], without: &[ComponentId]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = fnv::FnvHasher::default();
    
    for component_id in with {
        component_id.hash(&mut hasher);
    }
    
    0u8.hash(&mut hasher); // Separator
    
    for component_id in without {
        component_id.hash(&mut hasher);
    }
    
    hasher.finish()
}

/// Query configuration for commonly used queries
pub struct QueryConfig {
    pub with_components: Vec<ComponentId>,
    pub without_components: Vec<ComponentId>,
}

impl QueryConfig {
    pub fn new() -> Self {
        Self {
            with_components: Vec::new(),
            without_components: Vec::new(),
        }
    }
    
    pub fn with(mut self, component_id: ComponentId) -> Self {
        self.with_components.push(component_id);
        self
    }
    
    pub fn without(mut self, component_id: ComponentId) -> Self {
        self.without_components.push(component_id);
        self
    }
}
