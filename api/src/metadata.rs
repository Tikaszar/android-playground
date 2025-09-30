//! Module metadata for identification and dependency resolution

use crate::{ModuleType, Dependency};

/// Module metadata for identification and dependency resolution
pub struct ModuleMetadata {
    /// Module name (e.g., "core/ecs", "systems/webgl")
    pub name: &'static str,
    /// Semantic version (e.g., "1.0.0")
    pub version: &'static str,
    /// Module type for categorization
    pub module_type: ModuleType,
    /// Module dependencies with version requirements
    pub dependencies: &'static [Dependency],
    /// Feature flags this module provides
    pub features: &'static [&'static str],
}