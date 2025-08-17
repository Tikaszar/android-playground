use crate::error::LogicResult;
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

/// Entity handle with generation tracking for safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: Uuid,
    pub generation: u32,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            generation: 0,
        }
    }
    
    pub fn with_id(id: Uuid) -> Self {
        Self { id, generation: 0 }
    }
}

/// Entity builder for batch spawning
pub struct EntityBuilder {
    components: Vec<Box<dyn std::any::Any + Send + Sync>>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
    
    pub fn with<T: 'static + Send + Sync>(mut self, component: T) -> Self {
        self.components.push(Box::new(component));
        self
    }
    
    pub fn build(self) -> Vec<Box<dyn std::any::Any + Send + Sync>> {
        self.components
    }
}

/// Entity manager with generation tracking
pub struct EntityManager {
    alive: Arc<RwLock<fnv::FnvHashSet<Entity>>>,
    generations: Arc<RwLock<fnv::FnvHashMap<Uuid, u32>>>,
    recycled: Arc<RwLock<Vec<Uuid>>>,
}

use fnv::{FnvHashMap, FnvHashSet};

impl EntityManager {
    pub fn new() -> Self {
        Self {
            alive: Arc::new(RwLock::new(FnvHashSet::default())),
            generations: Arc::new(RwLock::new(FnvHashMap::default())),
            recycled: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn create(&self) -> Entity {
        let mut recycled = self.recycled.write();
        let id = if let Some(id) = recycled.pop() {
            id
        } else {
            Uuid::new_v4()
        };
        
        let mut generations = self.generations.write();
        let generation = generations.entry(id).or_insert(0);
        *generation = generation.wrapping_add(1);
        
        let entity = Entity {
            id,
            generation: *generation,
        };
        
        self.alive.write().insert(entity);
        entity
    }
    
    pub fn create_batch(&self, count: usize) -> Vec<Entity> {
        let mut entities = Vec::with_capacity(count);
        for _ in 0..count {
            entities.push(self.create());
        }
        entities
    }
    
    pub fn destroy(&self, entity: Entity) -> LogicResult<()> {
        let mut alive = self.alive.write();
        if !alive.remove(&entity) {
            return Err(crate::error::LogicError::EntityNotFound(entity.id));
        }
        
        self.recycled.write().push(entity.id);
        Ok(())
    }
    
    pub fn destroy_batch(&self, entities: &[Entity]) -> LogicResult<()> {
        let mut alive = self.alive.write();
        let mut recycled = self.recycled.write();
        
        for entity in entities {
            if !alive.remove(entity) {
                return Err(crate::error::LogicError::EntityNotFound(entity.id));
            }
            recycled.push(entity.id);
        }
        
        Ok(())
    }
    
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive.read().contains(&entity)
    }
    
    pub fn count(&self) -> usize {
        self.alive.read().len()
    }
    
    pub fn clear(&self) {
        let mut alive = self.alive.write();
        let mut recycled = self.recycled.write();
        
        for entity in alive.iter() {
            recycled.push(entity.id);
        }
        
        alive.clear();
    }
}