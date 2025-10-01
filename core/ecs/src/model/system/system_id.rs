//! System ID type

use serde::{Serialize, Deserialize};

/// System ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystemId(pub u32);

impl SystemId {
    /// Create a new system ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Create a null/invalid system ID
    pub fn null() -> Self {
        Self(u32::MAX)
    }

    /// Check if this is a null system ID
    pub fn is_null(&self) -> bool {
        self.0 == u32::MAX
    }
}

impl std::fmt::Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "System({})", self.0)
    }
}

impl From<u32> for SystemId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<SystemId> for u32 {
    fn from(id: SystemId) -> Self {
        id.0
    }
}
