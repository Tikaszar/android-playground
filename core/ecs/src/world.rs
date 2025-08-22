use std::collections::{HashMap, VecDeque};
use playground_core_types::{Shared, shared};
use crate::entity::{EntityId, EntityAllocator};
use crate::component::{Component, ComponentId, ComponentRegistry, ComponentBox};
use crate::storage::{ComponentStorage, SparseStorage, DenseStorage, StorageType};
use crate::query::{Query, QueryBuilder};
use crate::error::{EcsError, EcsResult};
use crate::messaging::{MessageBus, ChannelId};
use bytes::Bytes;

pub struct MemoryStats {
    pub total_entities: usize,
    pub total_components: usize,
    pub pool_usage: usize,
    pub pool_limit: usize,
    pub growth_rate: f32,
}

pub struct GarbageCollector {
    dead_entities: Shared<VecDeque<EntityId>>,
    frame_budget_ms: u64,
    enabled: bool,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            dead_entities: shared(VecDeque::new()),
            frame_budget_ms: 2,
            enabled: true,
        }
    }
    
    pub async fn queue_for_collection(&self, entity: EntityId) {
        self.dead_entities.write().await.push_back(entity);
    }
    
    pub async fn collect_incremental(&self, world: &World) -> EcsResult<usize> {
        if !self.enabled {
            return Ok(0);
        }
        
        let start = std::time::Instant::now();
        let mut collected = 0;
        
        while start.elapsed().as_millis() < self.frame_budget_ms as u128 {
            let entity = {
                let mut dead = self.dead_entities.write().await;
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
    entities: Shared<HashMap<EntityId, Vec<ComponentId>>>,
    storages: Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>,
    allocator: Shared<EntityAllocator>,
    registry: Shared<ComponentRegistry>,
    gc: Shared<GarbageCollector>,
    memory_stats: Shared<MemoryStats>,
    message_bus: Shared<MessageBus>,
}

impl World {
    pub fn new() -> Self {
        Self::with_registry(shared(ComponentRegistry::new()))
    }
    
    pub fn with_registry(registry: Shared<ComponentRegistry>) -> Self {
        Self {
            entities: shared(HashMap::new()),
            storages: shared(HashMap::new()),
            allocator: shared(EntityAllocator::new()),
            registry,
            gc: shared(GarbageCollector::new()),
            memory_stats: shared(MemoryStats {
                total_entities: 0,
                total_components: 0,
                pool_usage: 0,
                pool_limit: 100 * 1024 * 1024,
                growth_rate: 0.0,
            }),
            message_bus: shared(MessageBus::new()),
        }
    }
    
    pub async fn register_component<T: Component>(&self) -> EcsResult<()> {
        self.registry.read().await.register::<T>().await?;
        
        let component_id = T::component_id();
        if !self.storages.read().await.contains_key(&component_id) {
            let storage: Box<dyn ComponentStorage> = Box::new(SparseStorage::new(component_id));
            self.storages.write().await.insert(component_id, storage);
        }
        
        Ok(())
    }
    
    pub async fn register_component_with_storage<T: Component>(&self, storage_type: StorageType) -> EcsResult<()> {
        self.registry.read().await.register::<T>().await?;
        
        let component_id = T::component_id();
        if !self.storages.read().await.contains_key(&component_id) {
            let storage: Box<dyn ComponentStorage> = match storage_type {
                StorageType::Dense => Box::new(DenseStorage::new(component_id)),
                StorageType::Sparse | StorageType::Pooled => Box::new(SparseStorage::new(component_id)),
            };
            self.storages.write().await.insert(component_id, storage);
        }
        
        Ok(())
    }
    
    pub async fn spawn_batch(&self, bundles: Vec<Vec<ComponentBox>>) -> EcsResult<Vec<EntityId>> {
        let count = bundles.len();
        let entities = self.allocator.read().await.allocate_batch(count).await;
        
        for (entity, bundle) in entities.iter().zip(bundles) {
            let mut component_ids = Vec::new();
            
            for component in bundle {
                let type_name = std::any::type_name_of_val(&*component);
                let component_id = self.registry.read().await.get_info_by_name(type_name).await
                    .ok_or_else(|| EcsError::ComponentNotRegistered(type_name.to_string()))?
                    .id;
                
                component_ids.push(component_id);
                
                if let Some(storage) = self.storages.read().await.get(&component_id) {
                    storage.insert(*entity, component).await?;
                } else {
                    return Err(EcsError::ComponentNotRegistered(type_name.to_string()));
                }
            }
            
            self.entities.write().await.insert(*entity, component_ids);
        }
        
        let mut stats = self.memory_stats.write().await;
        stats.total_entities += count;
        
        Ok(entities)
    }
    
    pub async fn despawn_batch(&self, entities: Vec<EntityId>) -> EcsResult<()> {
        for entity in entities {
            self.gc.read().await.queue_for_collection(entity).await;
        }
        Ok(())
    }
    
    async fn despawn_immediate(&self, entity: EntityId) -> EcsResult<()> {
        if let Some(component_ids) = self.entities.write().await.remove(&entity) {
            for component_id in component_ids {
                if let Some(storage) = self.storages.read().await.get(&component_id) {
                    let _ = storage.remove(entity).await;
                }
            }
            
            self.allocator.read().await.free(entity).await;
            
            let mut stats = self.memory_stats.write().await;
            stats.total_entities = stats.total_entities.saturating_sub(1);
        }
        
        Ok(())
    }
    
    pub async fn register_component_storage(&self, component_id: ComponentId, storage_type: StorageType) -> EcsResult<()> {
        let mut storages = self.storages.write().await;
        if !storages.contains_key(&component_id) {
            let storage: Box<dyn ComponentStorage> = match storage_type {
                StorageType::Dense => Box::new(DenseStorage::new(component_id)),
                StorageType::Sparse => Box::new(SparseStorage::new(component_id)),
                StorageType::Pooled => Box::new(SparseStorage::new(component_id)), // Use sparse for now
            };
            storages.insert(component_id, storage);
        }
        Ok(())
    }
    
    pub async fn add_component_raw(&self, entity: EntityId, component: ComponentBox, component_id: ComponentId) -> EcsResult<()> {
        if let Some(storage) = self.storages.read().await.get(&component_id) {
            storage.insert(entity, component).await?;
            
            if let Some(components) = self.entities.write().await.get_mut(&entity) {
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
        if let Some(storage) = self.storages.read().await.get(&component_id) {
            let component_box = storage.remove(entity).await?;
            
            if let Some(components) = self.entities.write().await.get_mut(&entity) {
                components.retain(|&id| id != component_id);
            }
            
            Ok(component_box)
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    pub async fn get_component_raw(&self, entity: EntityId, component_id: ComponentId) -> EcsResult<ComponentBox> {
        if let Some(storage) = self.storages.read().await.get(&component_id) {
            storage.get_raw(entity).await
        } else {
            Err(EcsError::ComponentNotRegistered(format!("{:?}", component_id)))
        }
    }
    
    pub async fn get_component<T: Component>(&self, entity: EntityId) -> EcsResult<T> {
        let component_box = self.get_component_raw(entity, T::component_id()).await?;
        // Deserialize from bytes
        let bytes = component_box.serialize().await?;
        T::deserialize(&bytes).await
    }
    
    pub async fn update_component<T: Component>(&self, entity: EntityId, updater: impl FnOnce(&mut T)) -> EcsResult<()> {
        // Get the component
        let component = self.get_component::<T>(entity).await?;
        
        // Apply the update
        let mut updated = component;
        updater(&mut updated);
        
        // Remove old and add new
        self.remove_component_raw(entity, T::component_id()).await?;
        let component_box = Box::new(updated) as ComponentBox;
        self.add_component_raw(entity, component_box, T::component_id()).await?;
        
        Ok(())
    }
    
    pub async fn has_component(&self, entity: EntityId, component_id: ComponentId) -> bool {
        if let Some(storage) = self.storages.read().await.get(&component_id) {
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
        self.gc.read().await.collect_incremental(self).await
    }
    
    pub async fn get_dirty_entities(&self) -> Vec<(EntityId, Vec<ComponentId>)> {
        let mut result: Vec<(EntityId, Vec<ComponentId>)> = Vec::new();
        
        for (component_id, storage) in self.storages.read().await.iter() {
            let component_id = *component_id;
            let storage = storage;
            
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
        for (_component_id, storage) in self.storages.read().await.iter() {
            storage.clear_dirty().await?;
        }
        Ok(())
    }
    
    pub async fn memory_stats(&self) -> MemoryStats {
        let stats = self.memory_stats.read().await;
        MemoryStats {
            total_entities: stats.total_entities,
            total_components: self.storages.read().await.len(),
            pool_usage: self.registry.read().await.current_pool_usage().await,
            pool_limit: self.registry.read().await.pool_limit(),
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
        self.entities.read().await.is_empty()
    }
    
    // Messaging API
    
    /// Publish a message to a channel
    pub async fn publish(&self, channel: ChannelId, message: impl Into<Bytes>) -> EcsResult<()> {
        self.message_bus.read().await.publish(channel, message.into()).await
    }
    
    /// Subscribe to a channel with a handler
    pub async fn subscribe(
        &self,
        channel: ChannelId,
        handler: std::sync::Arc<dyn crate::messaging::MessageHandler>,
    ) -> EcsResult<()> {
        self.message_bus.read().await.subscribe(channel, handler).await
    }
    
    /// Get the message bus for advanced operations
    pub fn message_bus(&self) -> Shared<MessageBus> {
        self.message_bus.clone()
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
                return Err(EcsError::SerializationFailed("Invalid data".into()));
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