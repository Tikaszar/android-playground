use crate::component_data::ComponentData;
use crate::entity::Entity;
use crate::error::LogicResult;
use fnv::FnvHashMap;
use playground_core_types::{Handle, handle, Shared, shared};
use tokio::sync::RwLock;
use std::any::TypeId;

/// Archetype represents a unique combination of component types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Archetype {
    pub id: u64,
    pub component_types: Handle<Vec<TypeId>>,
}

impl Archetype {
    pub fn new(mut component_types: Vec<TypeId>) -> Self {
        // Sort for consistent hashing
        component_types.sort_by_key(|t| format!("{:?}", t));
        
        // Generate ID from component types
        let mut hasher = fnv::FnvHasher::default();
        use std::hash::{Hash, Hasher};
        for type_id in &component_types {
            type_id.hash(&mut hasher);
        }
        let id = hasher.finish();
        
        Self {
            id,
            component_types: handle(component_types),
        }
    }
    
    pub fn id(&self) -> u64 {
        self.id
    }
    
    pub fn component_types(&self) -> &[TypeId] {
        &self.component_types
    }
    
    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.component_types.contains(&type_id)
    }
}

use std::hash::Hasher;

/// Storage for entities in an archetype (column-based for cache efficiency)
pub struct ArchetypeStorage {
    pub archetype: Archetype,
    pub entities: Vec<Entity>,
    entity_indices: FnvHashMap<Entity, usize>,
    component_columns: FnvHashMap<TypeId, ComponentColumn>,
}

/// Column storage for a single component type
struct ComponentColumn {
    data: Vec<ComponentData>,
}

impl ArchetypeStorage {
    pub fn new(archetype: Archetype) -> Self {
        let mut component_columns = FnvHashMap::default();
        for &type_id in archetype.component_types() {
            component_columns.insert(type_id, ComponentColumn { data: Vec::new() });
        }
        
        Self {
            archetype,
            entities: Vec::new(),
            entity_indices: FnvHashMap::default(),
            component_columns,
        }
    }
    
    pub fn add_entity(&mut self, entity: Entity, components: Vec<ComponentData>) -> LogicResult<()> {
        if self.entity_indices.contains_key(&entity) {
            return Err(crate::error::LogicError::EntityNotFound(entity.id));
        }
        
        let index = self.entities.len();
        self.entities.push(entity);
        self.entity_indices.insert(entity, index);
        
        // Add components to columns
        for component in components {
            let type_id = component.type_id();
            if let Some(column) = self.component_columns.get_mut(&type_id) {
                column.data.push(component);
            }
        }
        
        Ok(())
    }
    
    pub fn remove_entity(&mut self, entity: Entity) -> LogicResult<Vec<ComponentData>> {
        let index = self.entity_indices
            .remove(&entity)
            .ok_or(crate::error::LogicError::EntityNotFound(entity.id))?;
        
        // Swap remove from entities
        self.entities.swap_remove(index);
        
        // Update index of swapped entity
        if index < self.entities.len() {
            let swapped_entity = self.entities[index];
            self.entity_indices.insert(swapped_entity, index);
        }
        
        // Remove components from columns
        let mut components = Vec::new();
        for &type_id in self.archetype.component_types() {
            if let Some(column) = self.component_columns.get_mut(&type_id) {
                components.push(column.data.swap_remove(index));
            }
        }
        
        Ok(components)
    }
    
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        let index = self.entity_indices.get(&entity)?;
        let type_id = TypeId::of::<T>();
        let column = self.component_columns.get(&type_id)?;
        column.data.get(*index)?.downcast_ref::<T>()
    }
    
    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let index = self.entity_indices.get(&entity)?;
        let type_id = TypeId::of::<T>();
        let column = self.component_columns.get_mut(&type_id)?;
        column.data.get_mut(*index)?.downcast_mut::<T>()
    }
    
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
    
    pub fn len(&self) -> usize {
        self.entities.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

/// Graph of archetype transitions for fast component add/remove
pub struct ArchetypeGraph {
    archetypes: Arc<RwLock<FnvHashMap<u64, Arc<RwLock<ArchetypeStorage>>>>>,
    edges: Arc<RwLock<FnvHashMap<(u64, TypeId, bool), u64>>>, // (from_archetype, component_type, is_add) -> to_archetype
}

impl ArchetypeGraph {
    pub fn new() -> Self {
        Self {
            archetypes: Arc::new(RwLock::new(FnvHashMap::default())),
            edges: Arc::new(RwLock::new(FnvHashMap::default())),
        }
    }
    
    pub async fn get_or_create_archetype(&self, component_types: Vec<TypeId>) -> Arc<RwLock<ArchetypeStorage>> {
        let archetype = Archetype::new(component_types);
        let id = archetype.id();
        
        let mut archetypes = self.archetypes.write().await;
        archetypes
            .entry(id)
            .or_insert_with(|| Arc::new(RwLock::new(ArchetypeStorage::new(archetype))))
            .clone()
    }
    
    pub async fn get_archetype(&self, id: u64) -> Option<Arc<RwLock<ArchetypeStorage>>> {
        self.archetypes.read().await.get(&id).cloned()
    }
    
    pub async fn find_transition(&self, from: u64, component_type: TypeId, is_add: bool) -> Option<u64> {
        self.edges.read().await.get(&(from, component_type, is_add)).copied()
    }
    
    pub async fn add_transition(&self, from: u64, component_type: TypeId, is_add: bool, to: u64) {
        self.edges.write().await.insert((from, component_type, is_add), to);
    }
    
    pub async fn move_entity(
        &self,
        entity: Entity,
        from_archetype: u64,
        component_type: TypeId,
        is_add: bool,
        new_component: Option<Box<dyn std::any::Any + Send + Sync>>,
    ) -> LogicResult<u64> {
        // Get source archetype
        let from = self.get_archetype(from_archetype).await
            .ok_or(crate::error::LogicError::ArchetypeNotFound(from_archetype))?;
        
        // Calculate destination archetype
        let mut component_types: Vec<_> = from.read().await.archetype.component_types().to_vec();
        if is_add {
            if !component_types.contains(&component_type) {
                component_types.push(component_type);
            }
        } else {
            component_types.retain(|&t| t != component_type);
        }
        
        // Get or create destination archetype
        let to_archetype = Archetype::new(component_types.clone());
        let to_id = to_archetype.id();
        let to = self.get_or_create_archetype(component_types).await;
        
        // Move entity and components
        let mut from_write = from.write().await;
        let mut components = from_write.remove_entity(entity)?;
        
        // Add or remove component
        if is_add {
            if let Some(new_comp) = new_component {
                components.push(new_comp);
            }
        } else {
            // Remove component from list
            // Note: This is simplified, real implementation would track component positions
        }
        
        let mut to_write = to.write().await;
        to_write.add_entity(entity, components)?;
        
        // Cache transition
        self.add_transition(from_archetype, component_type, is_add, to_id).await;
        
        Ok(to_id)
    }
    
    pub async fn all_archetypes(&self) -> Vec<Shared<ArchetypeStorage>> {
        self.archetypes.read().await.values().cloned().collect()
    }
}