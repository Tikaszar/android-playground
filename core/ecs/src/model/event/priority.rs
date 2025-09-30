//! Event priority levels

use serde::{Serialize, Deserialize};

/// Event handler priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Highest priority - runs first
    Critical = 0,
    /// High priority
    High = 1,
    /// Normal priority (default)
    Normal = 2,
    /// Low priority
    Low = 3,
    /// Lowest priority - runs last
    Minimal = 4,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}