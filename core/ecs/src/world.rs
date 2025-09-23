//! Concrete World data structure for the ECS
//! 
//! The World is the central data container for all entities and components.
//! ALL logic is implemented in systems/ecs - this just holds the data.
//! 
//! This is like an abstract base class - it has the structure but no behavior.

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use playground_core_types::{Handle, handle, Shared, shared};
use bytes::Bytes;
use crate::{
    Component, ComponentId, ComponentData, EntityId, Generation,
    Entity,
    CoreError, CoreResult, VTable,
};

/// The concrete World struct - data fields only, no logic!
/// 
/// This is like an abstract base class in OOP - it defines the structure
/// but all the behavior is implemented in systems/ecs.
pub struct World {
    /// Entity generation tracking
    pub entities: Shared<HashMap<EntityId, Generation>>,
    
    /// Component storage: entity -> component_id -> component
    pub components: Shared<HashMap<EntityId, HashMap<ComponentId, Component>>>,
    
    /// The VTable for system dispatch
    pub vtable: VTable,
    
    /// Next entity ID counter
    pub next_entity_id: AtomicU32,
}

impl World {
    /// Create a new World instance - just data initialization, no logic!
    pub fn new() -> Handle<Self> {
        handle(Self {
            entities: shared(HashMap::new()),
            components: shared(HashMap::new()),
            vtable: VTable::new(),
            next_entity_id: AtomicU32::new(1),
        })
    }
    
    // Public API methods - these delegate to VTable, no logic here!
    
    /// Spawn a new entity - returns an Entity handle
    pub async fn spawn_entity(self: &Handle<Self>) -> CoreResult<Entity> {
        let response = self.vtable.send_command(
            "ecs.entity",
            "spawn".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to spawn entity".to_string())));
        }

        // Deserialize EntityId from response
        let payload = response.payload.ok_or(CoreError::UnexpectedResponse)?;
        let id: EntityId = bincode::deserialize(&payload)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        // Return an Entity handle with generation 1 (new entity)
        Ok(Entity {
            id,
            generation: Generation::new(),
            world: self.clone(),
        })
    }
    
    /// Despawn an entity using its handle
    pub async fn despawn_entity(&self, entity: &Entity) -> CoreResult<()> {
        self.despawn_entity_internal(entity.id).await
    }

    /// Internal despawn by ID
    pub(crate) async fn despawn_entity_internal(&self, entity: EntityId) -> CoreResult<()> {
        let payload = bincode::serialize(&entity)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "ecs.entity",
            "despawn".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to despawn entity".to_string())));
        }
        
        Ok(())
    }
    
    /// Add a component to an entity using its handle
    pub async fn add_component<T: ComponentData>(&self, entity: &Entity, component: T) -> CoreResult<()> {
        let comp = Component::new(component).await?;
        self.add_component_internal(entity.id, comp).await
    }

    /// Internal add component by ID
    pub(crate) async fn add_component_internal(&self, entity: EntityId, component: Component) -> CoreResult<()> {
        // Serialize entity and component data separately since Component doesn't derive Serialize
        let entity_bytes = bincode::serialize(&entity)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        let component_id_bytes = bincode::serialize(&component.component_id)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        // Combine all the data
        let mut payload = Vec::new();
        payload.extend_from_slice(&(entity_bytes.len() as u32).to_le_bytes());
        payload.extend_from_slice(&entity_bytes);
        payload.extend_from_slice(&(component_id_bytes.len() as u32).to_le_bytes());
        payload.extend_from_slice(&component_id_bytes);
        payload.extend_from_slice(&(component.data.len() as u32).to_le_bytes());
        payload.extend_from_slice(&component.data);
        payload.extend_from_slice(component.component_name.as_bytes());
        
        let response = self.vtable.send_command(
            "ecs.component",
            "add".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to add component".to_string())));
        }
        
        Ok(())
    }
    
    /// Remove a component from an entity using its handle
    pub async fn remove_component<T: ComponentData>(&self, entity: &Entity) -> CoreResult<()> {
        self.remove_component_internal(entity.id, T::component_id()).await
    }

    /// Internal remove component by ID
    pub(crate) async fn remove_component_internal(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<()> {
        let payload = bincode::serialize(&(entity, component_id))
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "ecs.component",
            "remove".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to remove component".to_string())));
        }
        
        Ok(())
    }
    
    /// Get a component from an entity using its handle
    pub async fn get_component<T: ComponentData>(&self, entity: &Entity) -> CoreResult<T> {
        let comp = self.get_component_internal(entity.id, T::component_id()).await?;
        comp.deserialize::<T>().await
    }

    /// Internal get component by ID
    pub(crate) async fn get_component_internal(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<Component> {
        let payload = bincode::serialize(&(entity, component_id))
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "ecs.component",
            "get".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Component not found".to_string())));
        }
        
        // The response contains the component data - we need to reconstruct the Component
        let payload = response.payload.ok_or(CoreError::UnexpectedResponse)?;
        
        // For now, return a simple Component with the data
        // The actual component reconstruction will be handled by systems/ecs
        Ok(Component {
            data: payload,
            component_id,
            component_name: String::new(), // Will be filled by systems/ecs
            size_hint: 0,
        })
    }
    
    /// Query entities - returns Entity handles
    pub async fn query(self: &Handle<Self>, required: Vec<ComponentId>, excluded: Vec<ComponentId>) -> CoreResult<Vec<Entity>> {
        let payload = bincode::serialize(&(required, excluded))
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        let response = self.vtable.send_command(
            "ecs.query",
            "execute".to_string(),
            Bytes::from(payload)
        ).await?;

        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Query failed".to_string())));
        }

        let payload = response.payload.ok_or(CoreError::UnexpectedResponse)?;
        let ids: Vec<EntityId> = bincode::deserialize(&payload)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        // Convert EntityIds to Entity handles
        Ok(ids.into_iter().map(|id| Entity {
            id,
            generation: Generation::new(), // Will be validated by systems/ecs
            world: self.clone(),
        }).collect())
    }

    /// Validate an entity's existence and generation
    pub(crate) async fn validate_entity(&self, id: EntityId, generation: Generation) -> CoreResult<bool> {
        let payload = bincode::serialize(&(id, generation))
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        let response = self.vtable.send_command(
            "ecs.entity",
            "validate".to_string(),
            Bytes::from(payload)
        ).await?;

        Ok(response.success)
    }

    /// Check if an entity has a component
    pub(crate) async fn has_component(&self, entity: EntityId, component_id: ComponentId) -> bool {
        let payload = bincode::serialize(&(entity, component_id))
            .ok();

        if let Some(payload) = payload {
            if let Ok(response) = self.vtable.send_command(
                "ecs.component",
                "has".to_string(),
                Bytes::from(payload)
            ).await {
                return response.success;
            }
        }

        false
    }
}