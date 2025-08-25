use std::sync::atomic::{AtomicU32, Ordering};
use std::fmt;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Generation(u32);

impl Generation {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn increment(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    index: u32,
    generation: Generation,
}

impl EntityId {
    pub fn new(index: u32, generation: Generation) -> Self {
        Self { index, generation }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn generation(&self) -> Generation {
        self.generation
    }

    pub fn null() -> Self {
        Self {
            index: u32::MAX,
            generation: Generation(0),
        }
    }

    pub fn is_null(&self) -> bool {
        self.index == u32::MAX
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}:{})", self.index, self.generation.0)
    }
}

#[derive(Debug)]
pub struct Entity {
    id: EntityId,
    alive: bool,
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self { id, alive: true }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn is_alive(&self) -> bool {
        self.alive
    }

    pub fn kill(&mut self) {
        self.alive = false;
    }
}

pub struct EntityAllocator {
    next_index: AtomicU32,
    free_list: Mutex<Vec<(u32, Generation)>>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            next_index: AtomicU32::new(0),
            free_list: Mutex::new(Vec::new()),
        }
    }

    pub async fn allocate(&self) -> EntityId {
        let mut free_list = self.free_list.lock().await;
        
        if let Some((index, old_gen)) = free_list.pop() {
            EntityId::new(index, old_gen.increment())
        } else {
            drop(free_list);
            let index = self.next_index.fetch_add(1, Ordering::Relaxed);
            let generation = Generation::new(0);
            EntityId::new(index, generation)
        }
    }

    pub async fn allocate_batch(&self, count: usize) -> Vec<EntityId> {
        let mut result = Vec::with_capacity(count);
        let mut free_list = self.free_list.lock().await;
        
        for _ in 0..count {
            if let Some((index, old_gen)) = free_list.pop() {
                result.push(EntityId::new(index, old_gen.increment()));
            } else {
                break;
            }
        }
        
        drop(free_list);
        
        let remaining = count - result.len();
        if remaining > 0 {
            let start_index = self.next_index.fetch_add(remaining as u32, Ordering::Relaxed);
            for i in 0..remaining {
                result.push(EntityId::new(start_index + i as u32, Generation::new(0)));
            }
        }
        
        result
    }

    pub async fn free(&self, id: EntityId) {
        let mut free_list = self.free_list.lock().await;
        free_list.push((id.index(), id.generation()));
    }

    pub async fn free_batch(&self, ids: Vec<EntityId>) {
        let mut free_list = self.free_list.lock().await;
        for id in ids {
            free_list.push((id.index(), id.generation()));
        }
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_entity_allocation() {
        let allocator = EntityAllocator::new();
        
        let id1 = allocator.allocate().await;
        let id2 = allocator.allocate().await;
        
        assert_ne!(id1, id2);
        assert_eq!(id1.index(), 0);
        assert_eq!(id2.index(), 1);
    }

    #[tokio::test]
    async fn test_entity_recycling() {
        let allocator = EntityAllocator::new();
        
        let id1 = allocator.allocate().await;
        allocator.free(id1).await;
        
        let id2 = allocator.allocate().await;
        assert_eq!(id1.index(), id2.index());
        assert_ne!(id1.generation(), id2.generation());
    }

    #[tokio::test]
    async fn test_batch_allocation() {
        let allocator = EntityAllocator::new();
        
        let ids = allocator.allocate_batch(100).await;
        assert_eq!(ids.len(), 100);
        
        for i in 0..100 {
            assert_eq!(ids[i].index(), i as u32);
        }
    }
}