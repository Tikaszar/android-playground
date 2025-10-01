//! System execution statistics data structure

use serde::{Deserialize, Serialize};

/// Statistics for system execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub execution_count: u64,
    pub total_time_ms: f64,
    pub average_time_ms: f64,
    pub last_execution_time_ms: f64,
}

impl Default for SystemStats {
    fn default() -> Self {
        Self {
            execution_count: 0,
            total_time_ms: 0.0,
            average_time_ms: 0.0,
            last_execution_time_ms: 0.0,
        }
    }
}
