//! Event ID type

use serde::{Serialize, Deserialize};

/// Event ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub u32);

impl EventId {
    /// Create a new event ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Common event IDs
    pub const BEFORE_ENTITY_SPAWN: Self = Self(1);
    pub const AFTER_ENTITY_SPAWN: Self = Self(2);
    pub const BEFORE_ENTITY_DESPAWN: Self = Self(3);
    pub const AFTER_ENTITY_DESPAWN: Self = Self(4);
    pub const BEFORE_COMPONENT_ADD: Self = Self(5);
    pub const AFTER_COMPONENT_ADD: Self = Self(6);
    pub const BEFORE_COMPONENT_REMOVE: Self = Self(7);
    pub const AFTER_COMPONENT_REMOVE: Self = Self(8);
    pub const BEFORE_SYSTEM_EXECUTE: Self = Self(9);
    pub const AFTER_SYSTEM_EXECUTE: Self = Self(10);
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event({})", self.0)
    }
}