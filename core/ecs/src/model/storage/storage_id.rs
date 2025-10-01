//! Storage ID type

use serde::{Serialize, Deserialize};

/// Storage ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StorageId(pub u32);

impl StorageId {
    /// Create a new storage ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Create a null/invalid storage ID
    pub fn null() -> Self {
        Self(u32::MAX)
    }

    /// Check if this is a null storage ID
    pub fn is_null(&self) -> bool {
        self.0 == u32::MAX
    }
}

impl std::fmt::Display for StorageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Storage({})", self.0)
    }
}

impl From<u32> for StorageId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<StorageId> for u32 {
    fn from(id: StorageId) -> Self {
        id.0
    }
}
