//! Query ID type

use serde::{Serialize, Deserialize};

/// Query ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QueryId(pub u32);

impl QueryId {
    /// Create a new query ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Create a null/invalid query ID
    pub fn null() -> Self {
        Self(u32::MAX)
    }

    /// Check if this is a null query ID
    pub fn is_null(&self) -> bool {
        self.0 == u32::MAX
    }
}

impl std::fmt::Display for QueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Query({})", self.0)
    }
}

impl From<u32> for QueryId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<QueryId> for u32 {
    fn from(id: QueryId) -> Self {
        id.0
    }
}
