//! Module declaration in Cargo.toml

use serde::{Deserialize, Serialize};

/// Module declaration in Cargo.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDeclaration {
    /// Name of the Core module
    pub name: String,

    /// Features required from this Core module
    pub features: Vec<String>,

    /// System modules that can provide this Core module
    /// (in priority order - first is preferred)
    pub systems: Vec<String>,
}