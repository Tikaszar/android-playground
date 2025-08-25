use crate::archetype::ArchetypeGraph;
use crate::component::{Component, ComponentData, ComponentId};
use crate::entity::Entity;
use crate::error::LogicResult;
use fnv::FnvHashMap;
use playground_core_types::{Handle, handle, Shared, shared};

/// Hybrid storage combining archetype (for iteration) and sparse (for random access)
pub struct HybridStorage {
    /// Archetype storage for efficient iteration
    archetype_graph: Handle<ArchetypeGraph>,
    
    /// Entity location index for fast lookup
    entity_locations: Shared<FnvHashMap<Entity, EntityLocation>>,
    
    /// Sparse storage for singleton/rare components
    sparse_components: Shared<FnvHashMap<ComponentId, SparseStorage>>,
    
    /// Threshold for moving components to sparse storage
    sparse_threshold: usize,
}

#[derive(Clone, Copy, Debug)]
struct EntityLocation {
    archetype_id: u64,
    in_sparse: bool,
}

struct SparseStorage {
    components: FnvHashMap<Entity, Component>,
}

impl HybridStorage {
    pub fn new() -> Self {
        Self {
            archetype_graph: handle(ArchetypeGraph::new()),
            entity_locations: shared(FnvHashMap::default()),
            sparse_components: shared(FnvHashMap::default()),
            sparse_threshold: 100, // Components on fewer than 100 entities go to sparse
        }
    }
    
    pub async fn spawn_entity(&self, entity: Entity, components: Vec<Component>) -> LogicResult<()> {
        // Separate dense and sparse components
        let mut dense_types = Vec::new();
        let mut dense_components = Vec::new();
        let mut sparse_components_list = Vec::new();
        
        for component in components {
            let type_id = component.component_id();
            if self.should_use_sparse(type_id) {
                sparse_components_list.push(component);
            } else {
                dense_types.push(type_id);
                dense_components.push(component);
            }
        }
        
        // Add to archetype storage
        if !dense_types.is_empty() {
            let archetype_storage = self.archetype_graph.get_or_create_archetype(dense_types.clone()).await;
            let archetype_id = archetype_storage.read().await.archetype.id;
            archetype_storage.write().await.add_entity(entity, dense_components)?;
            
            self.entity_locations.write().await.insert(entity, EntityLocation {
                archetype_id,
                in_sparse: false,
            });
        }
        
        // Add sparse components
        if !sparse_components_list.is_empty() {
            let mut sparse = self.sparse_components.write().await;
            for component in sparse_components_list {
                let type_id = component.component_id();
                sparse
                    .entry(type_id)
                    .or_insert_with(|| SparseStorage {
                        components: FnvHashMap::default(),
                    })
                    .components
                    .insert(entity, component);
            }
            
            // Update location if not in archetype
            if dense_types.is_empty() {
                self.entity_locations.write().await.insert(entity, EntityLocation {
                    archetype_id: 0,
                    in_sparse: true,
                });
            }
        }
        
        Ok(())
    }
    
    pub async fn despawn_entity(&self, entity: Entity) -> LogicResult<()> {
        let locations = self.entity_locations.read().await;
        let location = locations
            .get(&entity)
            .ok_or(crate::error::LogicError::EntityNotFound(entity.id))?;
        
        // Remove from archetype
        if location.archetype_id != 0 {
            if let Some(archetype) = self.archetype_graph.get_archetype(location.archetype_id).await {
                archetype.write().await.remove_entity(entity)?;
            }
        }
        
        // Remove from sparse storage
        if location.in_sparse {
            let mut sparse = self.sparse_components.write().await;
            for storage in sparse.values_mut() {
                storage.components.remove(&entity);
            }
        }
        
        drop(locations);
        self.entity_locations.write().await.remove(&entity);
        
        Ok(())
    }
    
    pub async fn add_component_with_id(
        &self,
        entity: Entity,
        component_id: ComponentId,
        component: Component,
    ) -> LogicResult<()> {
        // Check if should use sparse
        if self.should_use_sparse(component_id) {
            self.sparse_components
                .write().await
                .entry(component_id)
                .or_insert_with(|| SparseStorage {
                    components: FnvHashMap::default(),
                })
                .components
                .insert(entity, component);
            return Ok(());
        }
        
        // Move entity to new archetype
        let location = {
            let locations = self.entity_locations.read().await;
            locations
                .get(&entity)
                .ok_or(crate::error::LogicError::EntityNotFound(entity.id))?
                .clone()
        };
        
        let new_archetype_id = self.archetype_graph.move_entity(
            entity,
            location.archetype_id,
            component_id,
            true,
            Some(component),
        ).await?;
        
        self.entity_locations.write().await.insert(entity, EntityLocation {
            archetype_id: new_archetype_id,
            in_sparse: location.in_sparse,
        });
        
        Ok(())
    }
    
    pub async fn add_component<T: ComponentData + serde::Serialize + 'static + Send + Sync>(
        &self,
        entity: Entity,
        component: T,
    ) -> LogicResult<()> {
        let component_id = T::component_id();
        let component_data = Component::new(component).await?;
        self.add_component_with_id(entity, component_id, component_data).await
    }
    
    pub async fn remove_component_by_id(&self, entity: Entity, component_id: ComponentId) -> LogicResult<()> {
        // Check sparse storage first
        if let Some(storage) = self.sparse_components.write().await.get_mut(&component_id) {
            if storage.components.remove(&entity).is_some() {
                return Ok(());
            }
        }
        
        // Remove from archetype
        let location = {
            let locations = self.entity_locations.read().await;
            locations
                .get(&entity)
                .ok_or(crate::error::LogicError::EntityNotFound(entity.id))?
                .clone()
        };
        
        let new_archetype_id = self.archetype_graph.move_entity(
            entity,
            location.archetype_id,
            component_id,
            false,
            None,
        ).await?;
        
        self.entity_locations.write().await.insert(entity, EntityLocation {
            archetype_id: new_archetype_id,
            in_sparse: location.in_sparse,
        });
        
        Ok(())
    }
    
    pub async fn remove_component<T: ComponentData + 'static>(&self, entity: Entity) -> LogicResult<()> {
        let component_id = T::component_id();
        self.remove_component_by_id(entity, component_id).await
    }
    
    pub async fn has_component_by_id(&self, entity: Entity, component_id: ComponentId) -> bool {
        let locations = self.entity_locations.read().await;
        let Some(location) = locations.get(&entity) else {
            return false;
        };
        
        // Check sparse storage
        if let Some(storage) = self.sparse_components.read().await.get(&component_id) {
            if storage.components.contains_key(&entity) {
                return true;
            }
        }
        
        // Check archetype storage
        if location.archetype_id != 0 {
            if let Some(archetype) = self.archetype_graph.get_archetype(location.archetype_id).await {
                if archetype.read().await.archetype.has_component(component_id) {
                    return true;
                }
            }
        }
        
        false
    }
    
    pub async fn has_component<T: ComponentData + 'static>(&self, entity: Entity) -> bool {
        let component_id = T::component_id();
        self.has_component_by_id(entity, component_id).await
    }
    
    fn should_use_sparse(&self, _component_id: ComponentId) -> bool {
        // For now, use archetype for everything
        // Could track usage statistics in the future to make this smarter
        false
    }
    
    pub async fn iter_archetype_entities(&self) -> Vec<Entity> {
        let mut entities = Vec::new();
        for archetype in self.archetype_graph.all_archetypes().await {
            entities.extend_from_slice(archetype.read().await.entities());
        }
        entities
    }
}


/// Storage statistics for optimization
pub struct StorageStats {
    pub archetype_count: usize,
    pub entity_count: usize,
    pub sparse_component_count: usize,
    pub memory_usage: usize,
}

impl HybridStorage {
    pub async fn stats(&self) -> StorageStats {
        let archetypes = self.archetype_graph.all_archetypes().await;
        let archetype_count = archetypes.len();
        let entity_count = self.entity_locations.read().await.len();
        let sparse_component_count: usize = self.sparse_components
            .read().await
            .values()
            .map(|s| s.components.len())
            .sum();
        
        // Calculate approximate memory usage
        let memory_usage = archetype_count * std::mem::size_of::<ArchetypeGraph>()
            + entity_count * std::mem::size_of::<EntityLocation>()
            + sparse_component_count * std::mem::size_of::<Component>();
        
        StorageStats {
            archetype_count,
            entity_count,
            sparse_component_count,
            memory_usage,
        }
    }
}