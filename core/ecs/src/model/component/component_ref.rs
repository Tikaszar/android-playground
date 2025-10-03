//! Component weak reference

use std::sync::Weak;
use playground_modules_types::Handle;
use crate::model::{
    entity::EntityId,
    component::ComponentId,
    world::World,
};

/// A weak reference to a component on an entity
///
/// This is safe to store as it will become invalid when
/// either the entity is despawned or the component is removed.
#[derive(Clone, Debug)]
pub struct ComponentRef {
    pub entity_id: EntityId,
    pub component_id: ComponentId,
    pub world: Weak<World>,
}

impl ComponentRef {
    /// Create a new component reference
    pub fn new(entity_id: EntityId, component_id: ComponentId, world: Weak<World>) -> Self {
        Self {
            entity_id,
            component_id,
            world,
        }
    }

    /// Get the entity ID
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Get the component ID
    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    /// Check if the weak reference is still valid (world exists)
    pub fn is_valid(&self) -> bool {
        self.world.upgrade().is_some()
    }

    /// Try to get a strong reference to the world
    pub fn world(&self) -> Option<Handle<World>> {
        self.world.upgrade().map(Handle::from)
    }
}

impl PartialEq for ComponentRef {
    fn eq(&self, other: &Self) -> bool {
        self.entity_id == other.entity_id && self.component_id == other.component_id
    }
}

impl Eq for ComponentRef {}

/// Serialization support for ComponentRef
impl serde::Serialize for ComponentRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ComponentRef", 2)?;
        state.serialize_field("entity_id", &self.entity_id)?;
        state.serialize_field("component_id", &self.component_id)?;
        state.end()
    }
}

/// Deserialization creates an invalid reference (needs to be fixed up)
impl<'de> serde::Deserialize<'de> for ComponentRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct ComponentRefData {
            entity_id: EntityId,
            component_id: ComponentId,
        }

        let data = ComponentRefData::deserialize(deserializer)?;
        Ok(ComponentRef {
            entity_id: data.entity_id,
            component_id: data.component_id,
            world: Weak::new(), // Invalid weak reference - needs to be fixed up
        })
    }
}
