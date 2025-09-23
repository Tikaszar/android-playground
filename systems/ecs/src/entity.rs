//! Entity management for the unified ECS

use std::collections::VecDeque;
use playground_core_types::{Shared, shared};
use playground_core_ecs::{EntityId, Generation};

/// Entity allocator manages entity IDs and generations
pub struct EntityAllocator {
    next_id: Shared<u32>,
    free_list: Shared<VecDeque<u32>>,
    generations: Shared<Vec<u32>>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            next_id: shared(1), // Start at 1, 0 is reserved for invalid
            free_list: shared(VecDeque::new()),
            generations: shared(Vec::new()),
        }
    }
    
    /// Allocate a single entity ID
    pub async fn allocate(&self) -> EntityId {
        let id = {
            let mut free_list = self.free_list.write().await;
            if let Some(id) = free_list.pop_front() {
                id
            } else {
                let mut next = self.next_id.write().await;
                let id = *next;
                *next += 1;
                id
            }
        };

        // Ensure generations vector is large enough
        {
            let mut generations = self.generations.write().await;
            while generations.len() <= id as usize {
                generations.push(0);
            }
        }

        EntityId::new(id)
    }
    
    /// Allocate a batch of entity IDs
    pub async fn allocate_batch(&self, count: usize) -> Vec<EntityId> {
        let mut result = Vec::with_capacity(count);
        for _ in 0..count {
            result.push(self.allocate().await);
        }
        result
    }
    
    /// Free an entity ID for reuse
    pub async fn free(&self, entity: EntityId) {
        let id = entity.index();
        
        // Increment generation for this ID
        {
            let mut generations = self.generations.write().await;
            if (id as usize) < generations.len() {
                generations[id as usize] += 1;
            }
        }
        
        // Add to free list
        self.free_list.write().await.push_back(id);
    }
    
    /// Check if an entity ID is valid
    pub async fn is_valid(&self, entity: EntityId) -> bool {
        let id = entity.index();

        // Just check if the ID has been allocated
        let next_id = self.next_id.read().await;
        id < *next_id && !self.free_list.read().await.contains(&id)
    }
    
    /// Get statistics about the allocator
    pub async fn stats(&self) -> AllocatorStats {
        AllocatorStats {
            total_allocated: *self.next_id.read().await - 1,
            free_count: self.free_list.read().await.len(),
            generation_count: self.generations.read().await.len(),
        }
    }
}

/// Statistics about the entity allocator
pub struct AllocatorStats {
    pub total_allocated: u32,
    pub free_count: usize,
    pub generation_count: usize,
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}