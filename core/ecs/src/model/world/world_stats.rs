//! World statistics data structure

use serde::{Deserialize, Serialize};

/// Statistics about the world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStats {
    pub entity_count: usize,
    pub component_count: usize,
    pub system_count: usize,
    pub event_count: usize,
    pub storage_count: usize,
    pub query_count: usize,
    pub total_memory_bytes: usize,
}

impl Default for WorldStats {
    fn default() -> Self {
        Self {
            entity_count: 0,
            component_count: 0,
            system_count: 0,
            event_count: 0,
            storage_count: 0,
            query_count: 0,
            total_memory_bytes: 0,
        }
    }
}
