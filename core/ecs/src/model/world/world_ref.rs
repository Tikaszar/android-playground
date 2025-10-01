//! World weak reference

use std::sync::Weak;
use playground_core_types::Handle;
use crate::model::world::World;

/// A weak reference to the World
///
/// This is safe to store as it will become invalid
/// when the World is destroyed.
#[derive(Clone)]
pub struct WorldRef {
    pub world: Weak<World>,
}

impl WorldRef {
    /// Create a new world reference from a weak pointer
    pub fn new(world: Weak<World>) -> Self {
        Self { world }
    }

    /// Try to upgrade to a strong World handle
    pub fn upgrade(&self) -> Option<Handle<World>> {
        self.world.upgrade().map(Handle::from)
    }

    /// Check if the weak reference is still valid (world exists)
    pub fn is_valid(&self) -> bool {
        self.world.upgrade().is_some()
    }
}

impl std::fmt::Debug for WorldRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldRef")
            .field("valid", &self.is_valid())
            .finish()
    }
}

impl PartialEq for WorldRef {
    fn eq(&self, other: &Self) -> bool {
        // Compare pointer addresses
        self.world.as_ptr() == other.world.as_ptr()
    }
}

impl Eq for WorldRef {}
