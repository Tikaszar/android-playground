//! Entity handle types for safe entity references
//!
//! This module provides Entity (strong reference) and EntityRef (weak reference)
//! types that prevent dangling entity references and provide safe access patterns.

use std::sync::{Arc, Weak};
use crate::{
    EntityId, Generation, World, Component, ComponentData,
    CoreResult,
};
use playground_core_types::Handle;

/// A weak reference to an entity (like Rc::Weak)
///
/// This is safe to store in components as it will become invalid
/// when the entity is despawned.
#[derive(Clone, Debug)]
pub struct EntityRef {
    pub(crate) id: EntityId,
    pub(crate) generation: Generation,
    pub(crate) world: Weak<World>,
}

impl EntityRef {
    /// Check if this entity reference is still valid
    pub async fn is_valid(&self) -> bool {
        if let Some(world) = self.world.upgrade() {
            world.validate_entity(self.id, self.generation).await.unwrap_or(false)
        } else {
            false
        }
    }

    /// Try to upgrade to a strong Entity handle
    pub fn upgrade(&self) -> Option<Entity> {
        self.world.upgrade().map(|world| Entity {
            id: self.id,
            generation: self.generation,
            world: Handle::from(world),
        })
    }

    /// Get the entity ID (use with caution)
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Get the generation
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Try to get a component from this entity
    pub async fn get_component<T: ComponentData>(&self) -> Option<T> {
        if let Some(entity) = self.upgrade() {
            entity.get_component::<T>().await.ok()
        } else {
            None
        }
    }
}

/// Implement PartialEq for EntityRef - compares ID and generation
impl PartialEq for EntityRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.generation == other.generation
    }
}

impl Eq for EntityRef {}

/// A strong reference to an entity
///
/// This handle keeps the World alive and provides direct access to
/// the entity's components. Convert to EntityRef for storage in components.
#[derive(Clone)]
pub struct Entity {
    pub(crate) id: EntityId,
    pub(crate) generation: Generation,
    pub(crate) world: Handle<World>,
}

impl Entity {
    /// Get the entity ID (use with caution, prefer passing Entity handles)
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Get the generation
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Create a weak reference to this entity
    pub fn downgrade(&self) -> EntityRef {
        EntityRef {
            id: self.id,
            generation: self.generation,
            world: Arc::downgrade(&self.world),
        }
    }

    /// Check if this entity is still valid
    pub async fn is_valid(&self) -> bool {
        self.world.validate_entity(self.id, self.generation).await.unwrap_or(false)
    }

    /// Add a component to this entity
    pub async fn add_component<T: ComponentData>(&self, component: T) -> CoreResult<()> {
        let component = Component::new(component).await?;
        self.world.add_component_internal(self.id, component).await
    }

    /// Get a component from this entity
    pub async fn get_component<T: ComponentData>(&self) -> CoreResult<T> {
        let component = self.world.get_component_internal(self.id, T::component_id()).await?;
        component.deserialize::<T>().await
    }

    /// Remove a component from this entity
    pub async fn remove_component<T: ComponentData>(&self) -> CoreResult<()> {
        self.world.remove_component_internal(self.id, T::component_id()).await
    }

    /// Check if this entity has a component
    pub async fn has_component<T: ComponentData>(&self) -> bool {
        self.world.has_component(self.id, T::component_id()).await
    }

    /// Despawn this entity
    pub async fn despawn(self) -> CoreResult<()> {
        self.world.despawn_entity_internal(self.id).await
    }
}

/// Implement PartialEq for Entity - compares ID and generation
impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.generation == other.generation
    }
}

impl Eq for Entity {}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entity")
            .field("id", &self.id)
            .field("generation", &self.generation)
            .finish()
    }
}

/// Serialization support for EntityRef
impl serde::Serialize for EntityRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("EntityRef", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("generation", &self.generation)?;
        state.end()
    }
}

/// Note: Deserialization of EntityRef requires a World reference to be meaningful
/// This is a placeholder that creates an invalid reference
impl<'de> serde::Deserialize<'de> for EntityRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct EntityRefData {
            id: EntityId,
            generation: Generation,
        }

        let data = EntityRefData::deserialize(deserializer)?;
        Ok(EntityRef {
            id: data.id,
            generation: data.generation,
            world: Weak::new(), // Invalid weak reference - needs to be fixed up
        })
    }
}