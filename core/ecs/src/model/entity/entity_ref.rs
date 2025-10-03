//! Entity weak reference

use std::sync::Weak;
use playground_modules_types::Handle;
use crate::model::{
    entity::{Entity, EntityId, Generation},
    world::World,
};

/// A weak reference to an entity (like Arc::Weak)
///
/// This is safe to store in components as it will become invalid
/// when the entity is despawned.
#[derive(Clone, Debug)]
pub struct EntityRef {
    pub id: EntityId,
    pub generation: Generation,
    pub world: Weak<World>,
}

impl EntityRef {
    /// Get the entity ID
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

impl PartialEq for EntityRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.generation == other.generation
    }
}

impl Eq for EntityRef {}

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

/// Deserialization creates an invalid reference (needs to be fixed up)
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