//! Storage data structures for components
//! 
//! This just defines the data structures. ALL logic is in systems/ecs/storage_impl.rs
//! This is like an abstract base class - structure without behavior.

use std::collections::HashMap;
use playground_core_types::{Shared, shared};
use crate::{Component, EntityId};

/// Storage type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// Dense storage - good for common components
    Dense,
    /// Sparse storage - good for rare components
    Sparse,
}

/// Concrete component storage - data structure only, no logic!
/// 
/// All operations on this storage are implemented in systems/ecs.
/// This just holds the data fields.
pub struct ComponentStorage {
    /// Storage type
    pub storage_type: StorageType,
    
    /// Dense storage: entity index -> component
    pub dense: Option<Shared<Vec<Option<Component>>>>,
    
    /// Sparse storage: entity -> component
    pub sparse: Option<Shared<HashMap<EntityId, Component>>>,
}

impl ComponentStorage {
    /// Create new storage of the specified type - just data initialization
    pub fn new(storage_type: StorageType) -> Self {
        match storage_type {
            StorageType::Dense => Self {
                storage_type,
                dense: Some(shared(Vec::new())),
                sparse: None,
            },
            StorageType::Sparse => Self {
                storage_type,
                dense: None,
                sparse: Some(shared(HashMap::new())),
            },
        }
    }
}