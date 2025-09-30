//! Module metadata structure

use crate::module::dependency::ModuleDependency;
use serde::{Deserialize, Serialize};

/// Metadata for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    /// Module name (e.g., "playground-core-ecs")
    pub name: &'static str,

    /// Module version (semver)
    pub version: &'static str,

    /// Module description
    pub description: &'static str,

    /// Features this module provides
    pub features: &'static [&'static str],

    /// Dependencies on other modules
    pub dependencies: &'static [ModuleDependency],
}