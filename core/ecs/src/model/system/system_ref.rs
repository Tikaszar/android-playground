//! System weak reference

use std::sync::Weak;
use playground_core_types::Handle;
use crate::model::{
    system::{System, SystemId},
    query::QueryId,
    world::World,
};

/// A weak reference to system metadata
///
/// This is safe to store as it will become invalid
/// when the system is unregistered.
#[derive(Clone, Debug)]
pub struct SystemRef {
    pub id: SystemId,
    pub name: String,
    pub query: QueryId,
    pub dependencies: Vec<SystemId>,
    pub world: Weak<World>,
}

impl SystemRef {
    /// Get the system ID
    pub fn id(&self) -> SystemId {
        self.id
    }

    /// Get the system name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the query this system operates on
    pub fn query(&self) -> QueryId {
        self.query
    }

    /// Get the system dependencies
    pub fn dependencies(&self) -> &[SystemId] {
        &self.dependencies
    }

    /// Try to upgrade to a strong System handle
    pub fn upgrade(&self) -> Option<System> {
        self.world.upgrade().map(|world| System {
            id: self.id,
            name: self.name.clone(),
            query: self.query,
            dependencies: self.dependencies.clone(),
            world: Handle::from(world),
        })
    }

    /// Check if the weak reference is still valid (world exists)
    pub fn is_valid(&self) -> bool {
        self.world.upgrade().is_some()
    }
}

impl PartialEq for SystemRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SystemRef {}

/// Serialization support for SystemRef
impl serde::Serialize for SystemRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SystemRef", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("query", &self.query)?;
        state.serialize_field("dependencies", &self.dependencies)?;
        state.end()
    }
}

/// Deserialization creates an invalid reference (needs to be fixed up)
impl<'de> serde::Deserialize<'de> for SystemRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct SystemRefData {
            id: SystemId,
            name: String,
            query: QueryId,
            dependencies: Vec<SystemId>,
        }

        let data = SystemRefData::deserialize(deserializer)?;
        Ok(SystemRef {
            id: data.id,
            name: data.name,
            query: data.query,
            dependencies: data.dependencies,
            world: Weak::new(), // Invalid weak reference - needs to be fixed up
        })
    }
}
