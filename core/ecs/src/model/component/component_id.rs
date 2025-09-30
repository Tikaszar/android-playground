//! Component ID type

use serde::{Serialize, Deserialize};

/// Component ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u32);

impl ComponentId {
    /// Create a component ID from a type name hash
    pub fn from_type_name<T: 'static>() -> Self {
        let type_name = std::any::type_name::<T>();
        let hash = Self::hash_type_name(type_name);
        ComponentId(hash)
    }

    /// Hash a type name to create a component ID
    fn hash_type_name(name: &str) -> u32 {
        // Simple FNV-1a hash
        let mut hash = 2166136261u32;
        for byte in name.bytes() {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
        hash
    }
}