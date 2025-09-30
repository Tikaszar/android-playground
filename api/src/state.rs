//! Module lifecycle state

use serde::{Serialize, Deserialize};

/// Module lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleState {
    /// Module is loaded but not initialized
    Loaded,
    /// Module is initializing
    Initializing,
    /// Module is ready for use
    Ready,
    /// Module is being hot-reloaded
    Reloading,
    /// Module is shutting down
    ShuttingDown,
    /// Module encountered an error
    Error,
}