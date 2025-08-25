use crate::error::LogicResult;
use crate::component::Component;
use playground_core_types::{Shared, shared};
use uuid::Uuid;
use fnv::{FnvHashMap, FnvHashSet};
use serde::{Serialize, Deserialize};

/// Entity handle with generation tracking for safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    components: Vec<Component>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
    
    pub fn with(mut self, component: Component) -> Self {
        self.components.push(component);
        self
    }
    
    pub fn build(self) -> Vec<Component> {
        self.components
    }
}

/// Entity manager with generation tracking
pub struct EntityManager {
    alive: Shared<FnvHashSet<Entity>>,
    generations: Shared<FnvHashMap<Uuid, u32>>,
    recycled: Shared<Vec<Uuid>>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            alive: shared(FnvHashSet::default()),
            generations: shared(FnvHashMap::default()),
            recycled: shared(Vec::new()),
        }
    }
    
    pub async fn create(&self) -> Entity {
        let mut recycled = self.recycled.write().await;
        let id = if let Some(id) = recycled.pop() {
            id
        } else {
            Uuid::new_v4()
        };
        
        let mut generations = self.generations.write().await;
        let generation = generations.entry(id).or_insert(0);
        *generation = generation.wrapping_add(1);
        
        let entity = Entity {
            id,
            generation: *generation,
        };
        
        self.alive.write().await.insert(entity);
        entity
    }
    
    pub async fn create_batch(&self, count: usize) -> Vec<Entity> {
        let mut entities = Vec::with_capacity(count);
        for _ in 0..count {
            entities.push(self.create().await);
        }
        entities
    }
    
    pub async fn destroy(&self, entity: Entity) -> LogicResult<()> {
        let mut alive = self.alive.write().await;
        if !alive.remove(&entity) {
            return Err(crate::error::LogicError::EntityNotFound(entity.id));
        }
        
        self.recycled.write().await.push(entity.id);
        Ok(())
    }
    
    pub async fn destroy_batch(&self, entities: &[Entity]) -> LogicResult<()> {
        let mut alive = self.alive.write().await;
        let mut recycled = self.recycled.write().await;
        
        for entity in entities {
            if !alive.remove(entity) {
                return Err(crate::error::LogicError::EntityNotFound(entity.id));
            }
            recycled.push(entity.id);
        }
        
        Ok(())
    }
    
    pub async fn is_alive(&self, entity: Entity) -> bool {
        self.alive.read().await.contains(&entity)
    }
    
    pub async fn count(&self) -> usize {
        self.alive.read().await.len()
    }
    
    pub async fn clear(&self) {
        let mut alive = self.alive.write().await;
        let mut recycled = self.recycled.write().await;
        
        for entity in alive.iter() {
            recycled.push(entity.id);
        }
        
        alive.clear();
    }
}