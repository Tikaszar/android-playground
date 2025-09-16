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
    Component, ComponentId, EntityId, Generation,
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
    
    /// Spawn a new entity (delegated to systems/ecs via VTable)
    pub async fn spawn_entity(&self) -> CoreResult<EntityId> {
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
        bincode::deserialize(&payload)
            .map_err(|e| CoreError::SerializationError(e.to_string()))
    }
    
    /// Despawn an entity (delegated to systems/ecs via VTable)
    pub async fn despawn_entity(&self, entity: EntityId) -> CoreResult<()> {
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
    
    /// Add a component to an entity (delegated to systems/ecs via VTable)
    pub async fn add_component(&self, entity: EntityId, component: Component) -> CoreResult<()> {
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
    
    /// Remove a component from an entity (delegated to systems/ecs via VTable)
    pub async fn remove_component(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<()> {
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
    
    /// Get a component from an entity (delegated to systems/ecs via VTable)
    pub async fn get_component(&self, entity: EntityId, component_id: ComponentId) -> CoreResult<Component> {
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
    
    /// Query entities (delegated to systems/ecs via VTable)
    pub async fn query(&self, required: Vec<ComponentId>, excluded: Vec<ComponentId>) -> CoreResult<Vec<EntityId>> {
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
        bincode::deserialize(&payload)
            .map_err(|e| CoreError::SerializationError(e.to_string()))
    }
}