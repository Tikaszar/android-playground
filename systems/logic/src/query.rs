use crate::entity::Entity;
use crate::storage::HybridStorage;
use fnv::FnvHashMap;
use parking_lot::RwLock;
use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;

/// Query builder for ECS queries
pub struct QueryBuilder<'a> {
    storage: &'a HybridStorage,
    with_components: Vec<TypeId>,
    without_components: Vec<TypeId>,
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
    
    pub fn with<T: 'static>(mut self) -> Self {
        self.with_components.push(TypeId::of::<T>());
        self
    }
    
    pub fn without<T: 'static>(mut self) -> Self {
        self.without_components.push(TypeId::of::<T>());
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
    with_components: Vec<TypeId>,
    without_components: Vec<TypeId>,
    cached: bool,
    cache_id: Option<u64>,
}

impl<'a> Query<'a> {
    pub fn iter(&self) -> QueryIter<'a> {
        // If cached, try to get from cache
        if let Some(cache_id) = self.cache_id {
            if let Some(cached_entities) = QUERY_CACHE.get(cache_id) {
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
        for entity in self.storage.iter_archetype_entities() {
            if self.matches_entity(entity) {
                entities.push(entity);
            }
        }
        
        // Cache if requested
        if let Some(cache_id) = self.cache_id {
            QUERY_CACHE.insert(cache_id, entities.clone());
        }
        
        QueryIter {
            entities,
            current: 0,
            storage: self.storage,
        }
    }
    
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(Entity),
    {
        for entity in self.iter() {
            f(entity);
        }
    }
    
    pub fn par_for_each<F>(&self, f: F)
    where
        F: Fn(Entity) + Send + Sync,
    {
        use rayon::prelude::*;
        let entities: Vec<_> = self.iter().collect();
        entities.par_iter().for_each(|&entity| f(entity));
    }
    
    fn matches_entity(&self, entity: Entity) -> bool {
        // Check all required components are present
        for &type_id in &self.with_components {
            if !self.has_component(entity, type_id) {
                return false;
            }
        }
        
        // Check no excluded components are present
        for &type_id in &self.without_components {
            if self.has_component(entity, type_id) {
                return false;
            }
        }
        
        true
    }
    
    fn has_component(&self, entity: Entity, type_id: TypeId) -> bool {
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

/// Query cache for frequently used queries
lazy_static::lazy_static! {
    static ref QUERY_CACHE: QueryCache = QueryCache::new();
}

struct QueryCache {
    cache: Arc<RwLock<FnvHashMap<u64, Vec<Entity>>>>,
}

impl QueryCache {
    fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(FnvHashMap::default())),
        }
    }
    
    fn get(&self, id: u64) -> Option<Vec<Entity>> {
        self.cache.read().get(&id).cloned()
    }
    
    fn insert(&self, id: u64, entities: Vec<Entity>) {
        self.cache.write().insert(id, entities);
    }
    
    fn invalidate(&self, id: u64) {
        self.cache.write().remove(&id);
    }
    
    fn clear(&self) {
        self.cache.write().clear();
    }
}

fn generate_cache_id(with: &[TypeId], without: &[TypeId]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = fnv::FnvHasher::default();
    
    for type_id in with {
        type_id.hash(&mut hasher);
    }
    
    0u8.hash(&mut hasher); // Separator
    
    for type_id in without {
        type_id.hash(&mut hasher);
    }
    
    hasher.finish()
}

/// Typed query for compile-time safety
pub struct TypedQuery<T> {
    _phantom: PhantomData<T>,
}

/// Macro for creating typed queries
#[macro_export]
macro_rules! query {
    ($storage:expr, $($with:ty),+) => {{
        $storage.query()
            $(.with::<$with>())+
            .build()
    }};
    
    ($storage:expr, $($with:ty),+ ; without $($without:ty),+) => {{
        $storage.query()
            $(.with::<$with>())+
            $(.without::<$without>())+
            .build()
    }};
}

