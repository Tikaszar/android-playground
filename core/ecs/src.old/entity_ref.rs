//! Entity handle types for safe entity references
//!
//! This module provides Entity (strong reference) and EntityRef (weak reference)
//! types that prevent dangling entity references and provide safe access patterns.

use std::sync::{Arc, Weak};
use crate::{
    EntityId, Generation, World,
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
    /// Get the entity ID (use with caution)
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Get the generation
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Try to upgrade to a strong Entity handle
    pub fn upgrade(&self) -> Option<Entity> {
        self.world.upgrade().map(|world| Entity {
            id: self.id,
            generation: self.generation,
            world: Handle::from(world),
        })
    }

    /// Check if the weak reference is still valid (world exists)
    pub fn is_valid(&self) -> bool {
        self.world.upgrade().is_some()
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
/// This handle keeps the World alive and provides the entity ID
/// and generation. Operations are performed via the module system.
#[derive(Clone)]
pub struct Entity {
    pub(crate) id: EntityId,
    pub(crate) generation: Generation,
    pub(crate) world: Handle<World>,
}

impl Entity {
    /// Create a new entity handle (used by systems/ecs module)
    pub fn new(id: EntityId, generation: Generation, world: Handle<World>) -> Self {
        Self { id, generation, world }
    }

    /// Get the entity ID (use with caution, prefer passing Entity handles)
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Get the generation
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Get a reference to the world
    pub fn world(&self) -> &Handle<World> {
        &self.world
    }

    /// Create a weak reference to this entity
    pub fn downgrade(&self) -> EntityRef {
        EntityRef {
            id: self.id,
            generation: self.generation,
            world: Arc::downgrade(&self.world),
        }
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