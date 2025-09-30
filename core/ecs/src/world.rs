//! Concrete World data structure for the ECS
//!
//! The World is the central data container for all entities and components.
//! ALL logic is implemented in systems/ecs - this just holds the data.
//!
//! This is a data-only structure - all operations are handled by the module system.

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use playground_core_types::{Handle, handle, Shared, shared};
use crate::{
    Component, ComponentId, EntityId, Generation,
};

/// The concrete World struct - data fields only, no logic!
///
/// This is pure data - all behavior is implemented in systems/ecs module.
/// Operations are performed via module calls, not methods.
pub struct World {
    /// Entity generation tracking
    pub entities: Shared<HashMap<EntityId, Generation>>,

    /// Component storage: entity -> component_id -> component
    pub components: Shared<HashMap<EntityId, HashMap<ComponentId, Component>>>,

    /// Next entity ID counter
    pub next_entity_id: AtomicU32,
}

impl World {
    /// Create a new World instance - just data initialization, no logic!
    pub fn new() -> Handle<Self> {
        handle(Self {
            entities: shared(HashMap::new()),
            components: shared(HashMap::new()),
            next_entity_id: AtomicU32::new(1),
        })
    }
}

/// Global world instance for the application
/// This will be accessed by the systems/ecs module
use once_cell::sync::Lazy;

static WORLD_INSTANCE: Lazy<Handle<World>> = Lazy::new(|| World::new());

/// Get the global world instance
pub fn get_world() -> Handle<World> {
    WORLD_INSTANCE.clone()
}