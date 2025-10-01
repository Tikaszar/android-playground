//! Event priority levels

use serde::{Serialize, Deserialize};

/// Event handler priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Pre-event handler (can cancel the event)
    Pre = 0,
    /// Highest priority - runs first (for post-event)
    Critical = 1,
    /// High priority
    High = 2,
    /// Normal priority (default)
    Normal = 3,
    /// Low priority
    Low = 4,
    /// Lowest priority - runs last
    Minimal = 5,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}