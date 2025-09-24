//! Server statistics component

use crate::types::*;
use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};

/// Server statistics as an ECS component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatsComponent {
    pub stats: ServerStats,
}

impl_component_data!(ServerStatsComponent);

impl ServerStatsComponent {
    pub fn new() -> Self {
        let mut stats = ServerStats::default();
        stats.start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self { stats }
    }
}

impl Default for ServerStatsComponent {
    fn default() -> Self {
        Self::new()
    }
}