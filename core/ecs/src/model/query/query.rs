//! Query handle (strong reference)

use playground_modules_types::{Handle, ModelTrait, ModelId, ModelType, model_type_of};
use crate::model::{
    query::{QueryId, QueryFilter},
    world::World,
};

/// A strong reference to a query
///
/// This handle keeps the World alive and provides the query ID
/// and filter. Query operations are performed via the view API.
#[derive(Clone)]
pub struct Query {
    pub id: QueryId,
    pub filter: QueryFilter,
    #[allow(dead_code)]
    pub world: Handle<World>,
}

impl ModelTrait for Query {
    fn model_id(&self) -> ModelId {
        self.id.0 as u64  // Convert QueryId's u32 to u64 ModelId
    }

    fn model_type(&self) -> ModelType {
        model_type_of::<Query>()  // Runtime-generated, but deterministic
    }
}

impl Query {
    /// Create a new query handle
    pub fn new(id: QueryId, filter: QueryFilter, world: Handle<World>) -> Self {
        Self { id, filter, world }
    }

    /// Get the query ID
    pub fn id(&self) -> QueryId {
        self.id
    }

    /// Get the query filter
    pub fn filter(&self) -> &QueryFilter {
        &self.filter
    }

    /// Get a reference to the world
    pub fn world(&self) -> &Handle<World> {
        &self.world
    }

    /// Create a weak reference to this query
    pub fn downgrade(&self) -> super::QueryRef {
        super::QueryRef {
            id: self.id,
            filter: self.filter.clone(),
            world: std::sync::Arc::downgrade(&self.world),
        }
    }
}

impl PartialEq for Query {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Query {}

impl std::fmt::Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Query")
            .field("id", &self.id)
            .field("filter", &self.filter)
            .finish()
    }
}
