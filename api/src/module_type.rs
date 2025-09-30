//! Types of modules in the system

use serde::{Serialize, Deserialize};

/// Types of modules in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleType {
    /// Core modules define data structures and contracts
    Core,
    /// System modules implement core contracts
    System,
    /// Plugin modules add features and gameplay
    Plugin,
    /// App modules are complete applications
    App,
}