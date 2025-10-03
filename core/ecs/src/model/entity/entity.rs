//! Entity handle (strong reference)

use playground_modules_types::Handle;
use crate::model::{
    entity::{EntityId, Generation},
    world::World,
};

/// A strong reference to an entity
///
/// This handle keeps the World alive and provides the entity ID
/// and generation. Operations are performed via the view API.
#[derive(Clone)]
pub struct Entity {
    pub id: EntityId,
    pub generation: Generation,
    #[allow(dead_code)]
    pub world: Handle<World>,
}

impl Entity {
    /// Create a new entity handle
    pub fn new(id: EntityId, generation: Generation, world: Handle<World>) -> Self {
        Self { id, generation, world }
    }

    /// Get the entity ID
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Get the generation
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Get a reference to the world
    pub fn world(&self) -> &Handle<World> {
        &self.world
    }

    /// Create a weak reference to this entity
    pub fn downgrade(&self) -> super::EntityRef {
        super::EntityRef {
            id: self.id,
            generation: self.generation,
            world: std::sync::Arc::downgrade(&self.world),
        }
    }
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.generation == other.generation
    }
}

impl Eq for Entity {}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entity")
            .field("id", &self.id)
            .field("generation", &self.generation)
            .finish()
    }
}