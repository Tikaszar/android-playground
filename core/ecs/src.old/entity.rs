//! Entity ID type for internal use
//!
//! This module defines the internal EntityId type. Public APIs should use
//! Entity and EntityRef handle types instead.

use serde::{Serialize, Deserialize};

/// Internal entity identifier
///
/// This is used internally by the ECS. Public APIs should use Entity or EntityRef
/// handle types which include generation tracking for safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u32);

impl EntityId {
    /// Create a new entity ID
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    /// Get the index value
    pub fn index(&self) -> u32 {
        self.0
    }

    /// Create a null/invalid entity ID
    pub fn null() -> Self {
        Self(u32::MAX)
    }

    /// Check if this is a null entity ID
    pub fn is_null(&self) -> bool {
        self.0 == u32::MAX
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

impl From<u32> for EntityId {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl From<EntityId> for u32 {
    fn from(id: EntityId) -> Self {
        id.0
    }
}