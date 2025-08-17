use crate::archetype::{ArchetypeGraph, ArchetypeStorage};
use crate::entity::Entity;
use crate::error::LogicResult;
use fnv::FnvHashMap;
use parking_lot::RwLock;
use std::any::TypeId;
use std::sync::Arc;

/// Hybrid storage combining archetype (for iteration) and sparse (for random access)
pub struct HybridStorage {
    /// Archetype storage for efficient iteration
    archetype_graph: Arc<ArchetypeGraph>,
    
    /// Entity location index for fast lookup
    entity_locations: Arc<RwLock<FnvHashMap<Entity, EntityLocation>>>,
    
    /// Sparse storage for singleton/rare components
    sparse_components: Arc<RwLock<FnvHashMap<TypeId, SparseStorage>>>,
    
    /// Threshold for moving components to sparse storage
    sparse_threshold: usize,
}

#[derive(Clone, Copy, Debug)]
struct EntityLocation {
    archetype_id: u64,
    in_sparse: bool,
}

struct SparseStorage {
    components: FnvHashMap<Entity, Box<dyn std::any::Any + Send + Sync>>,
}

impl HybridStorage {
    pub fn new() -> Self {
        Self {
            archetype_graph: Arc::new(ArchetypeGraph::new()),
            entity_locations: Arc::new(RwLock::new(FnvHashMap::default())),
            sparse_components: Arc::new(RwLock::new(FnvHashMap::default())),
            sparse_threshold: 100, // Components on fewer than 100 entities go to sparse
        }
    }
    
    pub fn spawn_entity(&self, entity: Entity, components: Vec<(TypeId, Box<dyn std::any::Any + Send + Sync>)>) -> LogicResult<()> {
        // Separate dense and sparse components
        let mut dense_types = Vec::new();
        let mut dense_components = Vec::new();
        let mut sparse_components_list = Vec::new();
        
        for (type_id, component) in components {
            if self.should_use_sparse(type_id) {
                sparse_components_list.push((type_id, component));
            } else {
                dense_types.push(type_id);
                dense_components.push(component);
            }
        }
        
        // Add to archetype storage
        if !dense_types.is_empty() {
            let archetype_storage = self.archetype_graph.get_or_create_archetype(dense_types.clone());
            let archetype_id = archetype_storage.read().archetype.id;
            archetype_storage.write().add_entity(entity, dense_components)?;
            
            self.entity_locations.write().insert(entity, EntityLocation {
                archetype_id,
                in_sparse: false,
            });
        }
        
        // Add sparse components
        if !sparse_components_list.is_empty() {
            let mut sparse = self.sparse_components.write();
            for (type_id, component) in sparse_components_list {
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
                self.entity_locations.write().insert(entity, EntityLocation {
                    archetype_id: 0,
                    in_sparse: true,
                });
            }
        }
        
        Ok(())
    }
    
    pub fn despawn_entity(&self, entity: Entity) -> LogicResult<()> {
        let locations = self.entity_locations.read();
        let location = locations
            .get(&entity)
            .ok_or(crate::error::LogicError::EntityNotFound(entity.id))?;
        
        // Remove from archetype
        if location.archetype_id != 0 {
            if let Some(archetype) = self.archetype_graph.get_archetype(location.archetype_id) {
                archetype.write().remove_entity(entity)?;
            }
        }
        
        // Remove from sparse storage
        if location.in_sparse {
            let mut sparse = self.sparse_components.write();
            for storage in sparse.values_mut() {
                storage.components.remove(&entity);
            }
        }
        
        drop(locations);
        self.entity_locations.write().remove(&entity);
        
        Ok(())
    }
    
    pub fn add_component<T: 'static + Send + Sync>(
        &self,
        entity: Entity,
        component: T,
    ) -> LogicResult<()> {
        let type_id = TypeId::of::<T>();
        
        // Check if should use sparse
        if self.should_use_sparse(type_id) {
            self.sparse_components
                .write()
                .entry(type_id)
                .or_insert_with(|| SparseStorage {
                    components: FnvHashMap::default(),
                })
                .components
                .insert(entity, Box::new(component));
            return Ok(());
        }
        
        // Move entity to new archetype
        let location = {
            let locations = self.entity_locations.read();
            locations
                .get(&entity)
                .ok_or(crate::error::LogicError::EntityNotFound(entity.id))?
                .clone()
        };
        
        let new_archetype_id = self.archetype_graph.move_entity(
            entity,
            location.archetype_id,
            type_id,
            true,
            Some(Box::new(component)),
        )?;
        
        self.entity_locations.write().insert(entity, EntityLocation {
            archetype_id: new_archetype_id,
            in_sparse: location.in_sparse,
        });
        
        Ok(())
    }
    
    pub fn remove_component<T: 'static>(&self, entity: Entity) -> LogicResult<()> {
        let type_id = TypeId::of::<T>();
        
        // Check sparse storage first
        if let Some(storage) = self.sparse_components.write().get_mut(&type_id) {
            if storage.components.remove(&entity).is_some() {
                return Ok(());
            }
        }
        
        // Remove from archetype
        let location = {
            let locations = self.entity_locations.read();
            locations
                .get(&entity)
                .ok_or(crate::error::LogicError::EntityNotFound(entity.id))?
                .clone()
        };
        
        let new_archetype_id = self.archetype_graph.move_entity(
            entity,
            location.archetype_id,
            type_id,
            false,
            None,
        )?;
        
        self.entity_locations.write().insert(entity, EntityLocation {
            archetype_id: new_archetype_id,
            in_sparse: location.in_sparse,
        });
        
        Ok(())
    }
    
    pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
        let locations = self.entity_locations.read();
        let Some(location) = locations.get(&entity) else {
            return false;
        };
        
        let type_id = TypeId::of::<T>();
        
        // Check sparse storage
        if let Some(storage) = self.sparse_components.read().get(&type_id) {
            if storage.components.contains_key(&entity) {
                return true;
            }
        }
        
        // Check archetype storage
        if location.archetype_id != 0 {
            if let Some(archetype) = self.archetype_graph.get_archetype(location.archetype_id) {
                if archetype.read().archetype.has_component(type_id) {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn should_use_sparse(&self, _type_id: TypeId) -> bool {
        // TODO: Track component usage statistics
        false // For now, use archetype for everything
    }
    
    pub fn iter_archetype_entities(&self) -> Vec<Entity> {
        let mut entities = Vec::new();
        for archetype in self.archetype_graph.all_archetypes() {
            entities.extend_from_slice(archetype.read().entities());
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
    pub fn stats(&self) -> StorageStats {
        let archetypes = self.archetype_graph.all_archetypes();
        let entity_count = self.entity_locations.read().len();
        let sparse_component_count: usize = self.sparse_components
            .read()
            .values()
            .map(|s| s.components.len())
            .sum();
        
        StorageStats {
            archetype_count: archetypes.len(),
            entity_count,
            sparse_component_count,
            memory_usage: 0, // TODO: Calculate actual memory usage
        }
    }
}