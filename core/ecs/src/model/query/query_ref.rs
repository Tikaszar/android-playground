//! Query weak reference

use std::sync::Weak;
use playground_core_types::Handle;
use crate::model::{
    query::{Query, QueryId, QueryFilter},
    world::World,
};

/// A weak reference to a query
///
/// This is safe to store as it will become invalid
/// when the query is no longer needed.
#[derive(Clone, Debug)]
pub struct QueryRef {
    pub id: QueryId,
    pub filter: QueryFilter,
    pub world: Weak<World>,
}

impl QueryRef {
    /// Get the query ID
    pub fn id(&self) -> QueryId {
        self.id
    }

    /// Get the query filter
    pub fn filter(&self) -> &QueryFilter {
        &self.filter
    }

    /// Try to upgrade to a strong Query handle
    pub fn upgrade(&self) -> Option<Query> {
        self.world.upgrade().map(|world| Query {
            id: self.id,
            filter: self.filter.clone(),
            world: Handle::from(world),
        })
    }

    /// Check if the weak reference is still valid (world exists)
    pub fn is_valid(&self) -> bool {
        self.world.upgrade().is_some()
    }
}

impl PartialEq for QueryRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for QueryRef {}

/// Serialization support for QueryRef
impl serde::Serialize for QueryRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("QueryRef", 1)?;
        state.serialize_field("id", &self.id)?;
        state.end()
    }
}

/// Deserialization creates an invalid reference (needs to be fixed up)
impl<'de> serde::Deserialize<'de> for QueryRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct QueryRefData {
            id: QueryId,
        }

        let data = QueryRefData::deserialize(deserializer)?;
        Ok(QueryRef {
            id: data.id,
            filter: QueryFilter::new(),
            world: Weak::new(), // Invalid weak reference - needs to be fixed up
        })
    }
}
