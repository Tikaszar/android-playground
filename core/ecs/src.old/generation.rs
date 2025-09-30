//! Generation tracking for entity validity
//!
//! Generations prevent entity ID reuse bugs by tracking how many times
//! an entity ID has been reused.

use serde::{Serialize, Deserialize};

/// Generation counter for an entity
///
/// This increments each time an entity ID is reused, preventing
/// dangling references from accessing new entities with recycled IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Generation(pub u32);

impl Generation {
    /// Create a new generation starting at 1
    pub fn new() -> Self {
        Generation(1)
    }

    /// Increment the generation
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
        // Skip 0 as it represents "invalid"
        if self.0 == 0 {
            self.0 = 1;
        }
    }

    /// Check if this generation is valid (non-zero)
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }

    /// Get an invalid generation
    pub fn invalid() -> Self {
        Generation(0)
    }
}

impl Default for Generation {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Generation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gen({})", self.0)
    }
}