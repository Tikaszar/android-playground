//! Storage handle (strong reference)

use playground_modules_types::{Handle, ModelTrait, ModelId, ModelType, model_type_of};
use crate::model::{
    storage::StorageId,
    world::World,
};

/// A strong reference to storage metadata
///
/// This handle keeps the World alive and provides storage metadata.
/// Actual I/O operations are performed via the view API.
#[derive(Clone)]
pub struct Storage {
    pub id: StorageId,
    pub path: String,
    pub format: String,
    #[allow(dead_code)]
    pub world: Handle<World>,
}

impl ModelTrait for Storage {
    fn model_id(&self) -> ModelId {
        self.id.0 as u64  // Convert StorageId's u32 to u64 ModelId
    }

    fn model_type(&self) -> ModelType {
        model_type_of::<Storage>()  // Runtime-generated, but deterministic
    }
}

impl Storage {
    /// Create a new storage handle
    pub fn new(id: StorageId, path: String, format: String, world: Handle<World>) -> Self {
        Self { id, path, format, world }
    }

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

    /// Get a reference to the world
    pub fn world(&self) -> &Handle<World> {
        &self.world
    }

    /// Create a weak reference to this storage
    pub fn downgrade(&self) -> super::StorageRef {
        super::StorageRef {
            id: self.id,
            path: self.path.clone(),
            format: self.format.clone(),
            world: std::sync::Arc::downgrade(&self.world),
        }
    }
}

impl PartialEq for Storage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Storage {}

impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage")
            .field("id", &self.id)
            .field("path", &self.path)
            .field("format", &self.format)
            .finish()
    }
}
