//! Query system for the ECS
//!
//! Provides entity querying using concrete types (no traits).

use crate::{ComponentId, Entity};

/// Concrete query builder struct
#[derive(Debug, Clone, Default)]
pub struct Query {
    /// Components that must be present
    pub required: Vec<ComponentId>,
    
    /// Components that must NOT be present
    pub excluded: Vec<ComponentId>,
    
    /// Optional tag filters
    pub with_tags: Vec<String>,
    pub without_tags: Vec<String>,
    
    /// Limit results
    pub limit: Option<usize>,
}

impl Query {
    /// Create a new empty query
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a required component (builder pattern)
    pub fn with_component(mut self, component_id: ComponentId) -> Self {
        self.required.push(component_id);
        self
    }
    
    /// Add an excluded component (builder pattern)
    pub fn without_component(mut self, component_id: ComponentId) -> Self {
        self.excluded.push(component_id);
        self
    }
    
    /// Add a required tag (builder pattern)
    pub fn with_tag(mut self, tag: String) -> Self {
        self.with_tags.push(tag);
        self
    }
    
    /// Add an excluded tag (builder pattern)
    pub fn without_tag(mut self, tag: String) -> Self {
        self.without_tags.push(tag);
        self
    }
    
    /// Set result limit (builder pattern)
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Check if an entity matches this query (given its components)
    pub fn matches(&self, entity_components: &[ComponentId]) -> bool {
        // Check all required components are present
        for required in &self.required {
            if !entity_components.contains(required) {
                return false;
            }
        }
        
        // Check no excluded components are present
        for excluded in &self.excluded {
            if entity_components.contains(excluded) {
                return false;
            }
        }
        
        true
    }
}

/// Result of a query
#[derive(Clone)]
pub struct QueryResult {
    /// Entities that matched the query
    pub entities: Vec<Entity>,

    /// Total count before limit was applied
    pub total_count: usize,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(entities: Vec<Entity>, total_count: usize) -> Self {
        Self { entities, total_count }
    }

    /// Check if the query returned any results
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Get the number of entities returned
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Iterate over the entities
    pub fn iter(&self) -> std::slice::Iter<Entity> {
        self.entities.iter()
    }

    /// Iterate over mutable entities
    pub fn iter_mut(&mut self) -> std::slice::IterMut<Entity> {
        self.entities.iter_mut()
    }
}

/// Query cache key for optimization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryKey {
    required: Vec<ComponentId>,
    excluded: Vec<ComponentId>,
}

impl QueryKey {
    /// Create a cache key from a query
    pub fn from_query(query: &Query) -> Self {
        let mut required = query.required.clone();
        required.sort_by_key(|id| id.0);
        
        let mut excluded = query.excluded.clone();
        excluded.sort_by_key(|id| id.0);
        
        Self { required, excluded }
    }
}