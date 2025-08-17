use std::sync::Arc;
use std::collections::VecDeque;
use dashmap::DashMap;
use parking_lot::RwLock;
use crate::entity::{EntityId, EntityAllocator};
use crate::component::{Component, ComponentId, ComponentRegistry, ComponentBox};
use crate::storage::{ComponentStorage, SparseStorage, DenseStorage, StorageType};
use crate::query::{Query, QueryBuilder};
use crate::error::{EcsError, EcsResult};

pub struct MemoryStats {
    pub total_entities: usize,
    pub total_components: usize,
    pub pool_usage: usize,
    pub pool_limit: usize,
    pub growth_rate: f32,
}

pub struct GarbageCollector {
    dead_entities: Arc<RwLock<VecDeque<EntityId>>>,
    frame_budget_ms: u64,
    enabled: bool,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            dead_entities: Arc::new(RwLock::new(VecDeque::new())),
            frame_budget_ms: 2,
            enabled: true,
        }
    }
    
    pub fn queue_for_collection(&self, entity: EntityId) {
        self.dead_entities.write().push_back(entity);
    }
    
    pub async fn collect_incremental(&self, world: &World) -> EcsResult<usize> {
        if !self.enabled {
            return Ok(0);
        }
        
        let start = std::time::Instant::now();
        let mut collected = 0;
        
        while start.elapsed().as_millis() < self.frame_budget_ms as u128 {
            let entity = {
                let mut dead = self.dead_entities.write();
                dead.pop_front()
            };
            
            if let Some(entity) = entity {
                world.despawn_immediate(entity).await?;
                collected += 1;
            } else {
                break;
            }
        }
        
        Ok(collected)
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn set_frame_budget(&mut self, ms: u64) {
        self.frame_budget_ms = ms;
    }
}

pub struct World {
    entities: Arc<DashMap<EntityId, Vec<ComponentId>>>,
    storages: Arc<DashMap<ComponentId, Arc<dyn ComponentStorage>>>,
    allocator: Arc<EntityAllocator>,
    registry: Arc<ComponentRegistry>,
    gc: Arc<RwLock<GarbageCollector>>,
    memory_stats: Arc<RwLock<MemoryStats>>,
}

impl World {
    pub fn new() -> Self {
        Self::with_registry(Arc::new(ComponentRegistry::new()))
    }
    
    pub fn with_registry(registry: Arc<ComponentRegistry>) -> Self {
        Self {
            entities: Arc::new(DashMap::new()),
            storages: Arc::new(DashMap::new()),
            allocator: Arc::new(EntityAllocator::new()),
            registry,
            gc: Arc::new(RwLock::new(GarbageCollector::new())),
            memory_stats: Arc::new(RwLock::new(MemoryStats {
                total_entities: 0,
                total_components: 0,
                pool_usage: 0,
                pool_limit: 100 * 1024 * 1024,
                growth_rate: 0.0,
            })),
        }
    }
    
    pub async fn register_component<T: Component>(&self) -> EcsResult<()> {
        self.registry.register::<T>().await?;
        
        let component_id = T::component_id();
        if !self.storages.contains_key(&component_id) {
            let storage: Arc<dyn ComponentStorage> = Arc::new(SparseStorage::new(component_id));
            self.storages.insert(component_id, storage);
        }
        
        Ok(())
    }
    
    pub async fn register_component_with_storage<T: Component>(&self, storage_type: StorageType) -> EcsResult<()> {
        self.registry.register::<T>().await?;
        
        let component_id = T::component_id();
        if !self.storages.contains_key(&component_id) {
            let storage: Arc<dyn ComponentStorage> = match storage_type {
                StorageType::Dense => Arc::new(DenseStorage::new(component_id)),
                StorageType::Sparse | StorageType::Pooled => Arc::new(SparseStorage::new(component_id)),
            };
            self.storages.insert(component_id, storage);
        }
        
        Ok(())
    }
    
    pub async fn spawn_batch(&self, bundles: Vec<Vec<ComponentBox>>) -> EcsResult<Vec<EntityId>> {
        let count = bundles.len();
        let entities = self.allocator.allocate_batch(count).await;
        
        for (entity, bundle) in entities.iter().zip(bundles) {
            let mut component_ids = Vec::new();
            
            for component in bundle {
                let type_name = std::any::type_name_of_val(&*component);
                let component_id = self.registry.get_info_by_name(type_name).await
                    .ok_or_else(|| EcsError::ComponentNotRegistered(type_name.to_string()))?
                    .id;
                
                component_ids.push(component_id);
                
                if let Some(storage) = self.storages.get(&component_id) {
                    storage.insert(*entity, component).await?;
                } else {
                    return Err(EcsError::ComponentNotRegistered(type_name.to_string()));
                }
            }
            
            self.entities.insert(*entity, component_ids);
        }
        
        let mut stats = self.memory_stats.write();
        stats.total_entities += count;
        
        Ok(entities)
    }
    
    pub async fn despawn_batch(&self, entities: Vec<EntityId>) -> EcsResult<()> {
        for entity in entities {
            self.gc.read().queue_for_collection(entity);
        }
        Ok(())
    }
    
    async fn despawn_immediate(&self, entity: EntityId) -> EcsResult<()> {
        if let Some((_, component_ids)) = self.entities.remove(&entity) {
            for component_id in component_ids {
                if let Some(storage) = self.storages.get(&component_id) {
                    let _ = storage.remove(entity).await;
                }
            }
            
            self.allocator.free(entity).await;
            
            let mut stats = self.memory_stats.write();
            stats.total_entities = stats.total_entities.saturating_sub(1);
        }
        
        Ok(())
    }
    
    pub async fn add_component_raw(&self, entity: EntityId, component: ComponentBox, component_id: ComponentId) -> EcsResult<()> {
        if let Some(storage) = self.storages.get(&component_id) {
            storage.insert(entity, component).await?;
            
            if let Some(mut components) = self.entities.get_mut(&entity) {
                if !components.contains(&component_id) {
                    components.push(component_id);
                }
            }
            
            Ok(())
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    pub async fn remove_component_raw(&self, entity: EntityId, component_id: ComponentId) -> EcsResult<ComponentBox> {
        if let Some(storage) = self.storages.get(&component_id) {
            let component_box = storage.remove(entity).await?;
            
            if let Some(mut components) = self.entities.get_mut(&entity) {
                components.retain(|&id| id != component_id);
            }
            
            Ok(component_box)
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    pub async fn get_component_raw(&self, entity: EntityId, component_id: ComponentId) -> EcsResult<ComponentBox> {
        if let Some(storage) = self.storages.get(&component_id) {
            storage.get_raw(entity).await
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    pub async fn has_component(&self, entity: EntityId, component_id: ComponentId) -> bool {
        if let Some(storage) = self.storages.get(&component_id) {
            storage.contains(entity).await
        } else {
            false
        }
    }
    
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new()
    }
    
    pub async fn execute_query(&self, query: &dyn Query) -> EcsResult<Vec<EntityId>> {
        query.execute(&self.storages).await
    }
    
    pub async fn run_gc(&self) -> EcsResult<usize> {
        self.gc.read().collect_incremental(self).await
    }
    
    pub async fn get_dirty_entities(&self) -> Vec<(EntityId, Vec<ComponentId>)> {
        let mut result: Vec<(EntityId, Vec<ComponentId>)> = Vec::new();
        
        for storage_entry in self.storages.iter() {
            let component_id = *storage_entry.key();
            let storage = storage_entry.value();
            
            for entity in storage.get_dirty().await {
                let exists = result.iter_mut().find(|(e, _)| *e == entity);
                if let Some((_, components)) = exists {
                    components.push(component_id);
                } else {
                    result.push((entity, vec![component_id]));
                }
            }
        }
        
        result
    }
    
    pub async fn clear_dirty(&self) -> EcsResult<()> {
        for storage_entry in self.storages.iter() {
            storage_entry.value().clear_dirty().await?;
        }
        Ok(())
    }
    
    pub async fn memory_stats(&self) -> MemoryStats {
        let stats = self.memory_stats.read();
        MemoryStats {
            total_entities: stats.total_entities,
            total_components: self.storages.len(),
            pool_usage: self.registry.current_pool_usage(),
            pool_limit: self.registry.pool_limit(),
            growth_rate: stats.growth_rate,
        }
    }
    
    pub async fn check_memory_pressure(&self) -> Option<String> {
        let stats = self.memory_stats().await;
        let usage_percent = (stats.pool_usage as f32 / stats.pool_limit as f32) * 100.0;
        
        if usage_percent > 90.0 {
            Some(format!("Critical: Memory usage at {:.1}%", usage_percent))
        } else if usage_percent > 75.0 {
            Some(format!("Warning: Memory usage at {:.1}%", usage_percent))
        } else if stats.growth_rate > 10.0 {
            Some(format!("Warning: Memory growing rapidly at {:.1}% per second", stats.growth_rate))
        } else {
            None
        }
    }
    
    pub async fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};
    use async_trait::async_trait;
    
    struct Position {
        x: f32,
        y: f32,
    }
    
    #[async_trait]
    impl Component for Position {
        async fn serialize(&self) -> EcsResult<Bytes> {
            let mut buf = BytesMut::new();
            buf.extend_from_slice(&self.x.to_le_bytes());
            buf.extend_from_slice(&self.y.to_le_bytes());
            Ok(buf.freeze())
        }
        
        async fn deserialize(bytes: &Bytes) -> EcsResult<Self> {
            if bytes.len() < 8 {
                return Err(EcsError::SerializationError("Invalid data".into()));
            }
            Ok(Self {
                x: f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                y: f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            })
        }
    }
    
    #[tokio::test]
    async fn test_world_spawn() {
        let world = World::new();
        world.register_component::<Position>().await.unwrap();
        
        let entities = world.spawn_batch(vec![
            vec![Box::new(Position { x: 0.0, y: 0.0 }) as ComponentBox],
            vec![Box::new(Position { x: 1.0, y: 1.0 }) as ComponentBox],
        ]).await.unwrap();
        
        assert_eq!(entities.len(), 2);
        assert!(!world.is_empty().await);
    }
}