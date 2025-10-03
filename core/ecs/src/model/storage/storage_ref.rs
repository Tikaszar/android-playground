//! Storage weak reference

use std::sync::Weak;
use playground_modules_types::Handle;
use crate::model::{
    storage::{Storage, StorageId},
    world::World,
};

/// A weak reference to storage metadata
///
/// This is safe to store as it will become invalid
/// when the storage is no longer needed.
#[derive(Clone, Debug)]
pub struct StorageRef {
    pub id: StorageId,
    pub path: String,
    pub format: String,
    pub world: Weak<World>,
}

impl StorageRef {
    /// Get the storage ID
    pub fn id(&self) -> StorageId {
        self.id
    }

    /// Get the storage path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the storage format
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Try to upgrade to a strong Storage handle
    pub fn upgrade(&self) -> Option<Storage> {
        self.world.upgrade().map(|world| Storage {
            id: self.id,
            path: self.path.clone(),
            format: self.format.clone(),
            world: Handle::from(world),
        })
    }

    /// Check if the weak reference is still valid (world exists)
    pub fn is_valid(&self) -> bool {
        self.world.upgrade().is_some()
    }
}

impl PartialEq for StorageRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for StorageRef {}

/// Serialization support for StorageRef
impl serde::Serialize for StorageRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("StorageRef", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("path", &self.path)?;
        state.serialize_field("format", &self.format)?;
        state.end()
    }
}

/// Deserialization creates an invalid reference (needs to be fixed up)
impl<'de> serde::Deserialize<'de> for StorageRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct StorageRefData {
            id: StorageId,
            path: String,
            format: String,
        }

        let data = StorageRefData::deserialize(deserializer)?;
        Ok(StorageRef {
            id: data.id,
            path: data.path,
            format: data.format,
            world: Weak::new(), // Invalid weak reference - needs to be fixed up
        })
    }
}
