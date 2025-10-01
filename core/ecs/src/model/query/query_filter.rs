//! Query filter for component matching

use crate::model::component::ComponentId;

/// Query filter describing which components to match
#[derive(Debug, Clone)]
pub struct QueryFilter {
    /// Components that must be present
    pub include: Vec<ComponentId>,

    /// Components that must NOT be present
    pub exclude: Vec<ComponentId>,
}

impl QueryFilter {
    /// Create a new empty query filter
    pub fn new() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
        }
    }

    /// Add a component that must be present
    pub fn with(mut self, component_id: ComponentId) -> Self {
        self.include.push(component_id);
        self
    }

    /// Add a component that must NOT be present
    pub fn without(mut self, component_id: ComponentId) -> Self {
        self.exclude.push(component_id);
        self
    }
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self::new()
    }
}
