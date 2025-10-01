//! System handle (strong reference)

use playground_core_types::Handle;
use crate::model::{
    system::SystemId,
    query::QueryId,
    world::World,
};

/// A strong reference to system metadata
///
/// This handle keeps the World alive and provides system metadata.
/// The actual system implementation lives in ViewModel (systems/ecs).
#[derive(Clone)]
pub struct System {
    pub id: SystemId,
    pub name: String,
    pub query: QueryId,
    pub dependencies: Vec<SystemId>,
    #[allow(dead_code)]
    pub world: Handle<World>,
}

impl System {
    /// Create a new system handle
    pub fn new(id: SystemId, name: String, query: QueryId, dependencies: Vec<SystemId>, world: Handle<World>) -> Self {
        Self { id, name, query, dependencies, world }
    }

    /// Get the system ID
    pub fn id(&self) -> SystemId {
        self.id
    }

    /// Get the system name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the query this system operates on
    pub fn query(&self) -> QueryId {
        self.query
    }

    /// Get the system dependencies
    pub fn dependencies(&self) -> &[SystemId] {
        &self.dependencies
    }

    /// Get a reference to the world
    pub fn world(&self) -> &Handle<World> {
        &self.world
    }

    /// Create a weak reference to this system
    pub fn downgrade(&self) -> super::SystemRef {
        super::SystemRef {
            id: self.id,
            name: self.name.clone(),
            query: self.query,
            dependencies: self.dependencies.clone(),
            world: std::sync::Arc::downgrade(&self.world),
        }
    }
}

impl PartialEq for System {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for System {}

impl std::fmt::Debug for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("System")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("query", &self.query)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}
